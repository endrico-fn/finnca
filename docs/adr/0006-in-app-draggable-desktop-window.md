# ADR 0006: In-App Draggable Desktop Window vs Native OS Multi-Window

## Status

Accepted

## Context

Dalam perancangan formulir pencatatan transaksi keuangan, transfer dana, dan dialog inspeksi di aplikasi desktop Finnca, tim menghadapi dua opsi arsitektur tampilan melayang (*floating window*):

1. **Native OS Multi-Window**: Menginstansiasi jendela sekunder sistem operasi via Tauri `WebviewWindow` terpisah (`tauri::WebviewWindowBuilder`).
2. **In-App Draggable Modal**: Membangun dialog melayang di dalam webview utama menggunakan Svelte 5 DOM (`DraggableModal.svelte`) dengan koordinat berbasis pointer dan akselerasi hardware.

Pada evaluasi performa di berbagai perangkat keras (khususnya laptop hemat daya dan arsitektur low-power x86/ARM), pembentukan native OS multi-window menimbulkan beberapa masalah kritis:
- **Cold-Start Latency (1–2 detik)**: Setiap instansiasi `WebviewWindow` baru memerlukan inisialisasi runtime webview OS baru (WebKitGTK di Linux, WebView2 di Windows), menyebabkan delay visual yang nyata dan mengurangi kenyamanan navigasi cepat.
- **Konsumsi Memori Berlebih**: Setiap webview sekunder mengalokasikan context renderer dan memori grafis independen (rata-rata 80–150 MB RAM per jendela tambahan).
- **CSS & Rendering Drift**: Dekorasi jendela OS, frame border, dan event window focus berbeda drastis antara Windows DWM, Linux X11, dan Linux Wayland compositor.
- **Kompleksitas State Synchronization**: Diperlukan IPC event bus lintas webview untuk menjaga reaktivitas saldo akun dan draf transaksi agar selalu sinkron.

## Decision

Kami menetapkan arsitektur **In-App Draggable Modal** (`DraggableModal.svelte`) sebagai standar tunggal untuk seluruh dialog aksi dan formulir input melayang di Finnca:

1. **Latensi Nol (Instant 0ms Launch)**:
   Dialog dirender langsung di dalam hierarki DOM webview utama yang telah siap, sehingga peluncuran formulir transaksi terjadi secara instan (0 milidetik).

2. **Pointer Capture Hygiene & RAF Coordinate Pipeline**:
   - Seluruh pergerakan drag dikalkulasikan melalui `requestAnimationFrame` (RAF) guna mencegah *event flooding* dan mempertahankan 60–120 FPS tanpa frame drop.
   - Event listener pointer capture dilepas secara eksplisit (`releasePointerCapture`) saat `pointerup` dan saat komponen di-unmount, mencegah pointer terjebak (*pointer lock leak*).

3. **Backdrop Trapping & Click Ergonomics**:
   - Status seret (`hasDragged`) di-reset secara higienis ketika interaksi pointer selesai, sehingga pengguna dapat menutup modal hanya dengan satu klik di *backdrop* jika jendela tidak digeser.
   - Posisi seret dibatasi (*clamped*) di dalam viewport aplikasi agar modal tidak dapat tersesat keluar dari layar yang dapat dijangkau pengguna.

4. **Zero CSS Drift Across Operating Systems**:
   - Menggunakan token semantik Tailwind v4 (`bg-bg-card`, `border-line`, `rounded-none`, `font-proto`, `font-aux`) yang menjamin konsistensi estetika *utilitarian-brutalist industrial tech* secara identik di Linux dan Windows.
   - Tidak ada artefak border bawaan OS atau inkonsistensi tema dekoratif.

5. **Integrated Focus Management & Keyboard Trapping**:
   - Navigasi keyboard (`Tab`, `Escape`, `Enter`) terisolasi di dalam container modal tanpa memerlukan protokol koordinasi window focus level OS.

## Consequences

- **Positif:** Launch instan tanpa jeda cold-start 1–2 detik pada CPU hemat daya.
- **Positif:** Penghematan RAM signifikan karena hanya ada satu proses renderer webview aktif.
- **Positif:** Konsistensi visual dan tipografi 100% identik di seluruh sistem operasi.
- **Positif:** Bebas dari race condition sinkronisasi window state dan koordinasi multi-window IPC.
- **Trade-off:** Jendela formulir terbatas pada batas viewport aplikasi utama (`#app-root`) dan tidak dapat dipindahkan keluar ke monitor fisik sekunder secara independen. Untuk kasus penggunaan pencatatan keuangan pribadi harian, sifat modal yang transien dan fokus tinggi menjadikan trade-off ini optimal.
