import { browser } from '$app/environment';

export type JobStatus = 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED';

export interface Job {
  id: string;
  type: string;
  payload: Record<string, unknown>;
  status: JobStatus;
  progress: number; // 0 to 100
  retryCount: number;
  error?: string;
  createdAt: number;
  updatedAt: number;
}

type JobHandler = (job: Job, updateProgress: (p: number) => void) => Promise<void>;

class JobManager {
  jobs = $state<Job[]>([]);
  private handlers = new Map<string, JobHandler>();
  private isProcessing = false;

  constructor() {
    if (browser) {
      this.loadFromStorage();
      // Pemulihan Durable Execution: Reset job yang 'RUNNING' saat aplikasi crash menjadi 'PENDING'
      let needsSave = false;
      for (const job of this.jobs) {
        if (job.status === 'RUNNING') {
          job.status = 'PENDING';
          job.updatedAt = Date.now();
          needsSave = true;
        }
      }
      if (needsSave) this.saveToStorage();
    }
  }

  registerHandler(type: string, handler: JobHandler) {
    this.handlers.set(type, handler);
    // Jalankan antrean jika ada job tertunda setelah handler didaftarkan (Misal: setelah restart)
    if (browser) {
      this.processQueue();
    }
  }

  addJob(type: string, payload: Record<string, unknown>) {
    const newJob: Job = {
      id: crypto.randomUUID(),
      type,
      payload,
      status: 'PENDING',
      progress: 0,
      retryCount: 0,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    this.jobs.push(newJob);
    this.saveToStorage();
    this.processQueue();
  }

  async processQueue() {
    if (this.isProcessing) return;
    this.isProcessing = true;

    try {
      while (true) {
        const nextJob = this.jobs.find((j) => j.status === 'PENDING');
        if (!nextJob) break;

        const handler = this.handlers.get(nextJob.type);
        if (!handler) {
          // Abaikan dulu jika handler belum di-register, mungkin modul lain belum me-load
          break;
        }

        nextJob.status = 'RUNNING';
        nextJob.updatedAt = Date.now();
        this.saveToStorage();

        try {
          await handler(nextJob, (p) => {
            nextJob.progress = p;
            this.saveToStorage();
          });
          nextJob.status = 'COMPLETED';
          nextJob.progress = 100;
        } catch (err: unknown) {
          nextJob.retryCount++;
          if (nextJob.retryCount >= 3) {
            nextJob.status = 'FAILED';
            nextJob.error = err instanceof Error ? err.message : String(err);
          } else {
            nextJob.status = 'PENDING'; // Auto-retry
          }
        } finally {
          nextJob.updatedAt = Date.now();
          this.saveToStorage();
        }
      }
    } finally {
      this.isProcessing = false;
    }
  }

  private saveToStorage() {
    if (!browser) return;
    localStorage.setItem('finnca_jobs', JSON.stringify(this.jobs));
  }

  private loadFromStorage() {
    if (!browser) return;
    try {
      const data = localStorage.getItem('finnca_jobs');
      if (data) {
        this.jobs = JSON.parse(data);
      }
    } catch (e) {
      console.error('Failed to load jobs', e);
    }
  }

  clearCompleted() {
    this.jobs = this.jobs.filter((j) => j.status !== 'COMPLETED' && j.status !== 'FAILED');
    this.saveToStorage();
  }
}

export const jobManager = new JobManager();
