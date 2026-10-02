use super::models::{Account, AccountType};
use super::repository;
use crate::shared::{generate_id, AppError};
use rusqlite::Connection;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RootAccountDefinition {
    pub code: &'static str,
    pub name: &'static str,
    pub account_type: AccountType,
    pub description: &'static str,
}

pub const ROOT_PLACEHOLDER_ACCOUNTS: [RootAccountDefinition; 5] = [
    RootAccountDefinition {
        code: "1000",
        name: "Assets",
        account_type: AccountType::Asset,
        description: "Root placeholder account for Assets",
    },
    RootAccountDefinition {
        code: "2000",
        name: "Liabilities",
        account_type: AccountType::Liability,
        description: "Root placeholder account for Liabilities",
    },
    RootAccountDefinition {
        code: "3000",
        name: "Equity",
        account_type: AccountType::Equity,
        description: "Root placeholder account for Equity",
    },
    RootAccountDefinition {
        code: "4000",
        name: "Income",
        account_type: AccountType::Income,
        description: "Root placeholder account for Income",
    },
    RootAccountDefinition {
        code: "5000",
        name: "Expenses",
        account_type: AccountType::Expense,
        description: "Root placeholder account for Expenses",
    },
];

#[derive(Debug, Clone)]
pub struct ComprehensiveAccountDef {
    pub code: &'static str,
    pub name_en: &'static str,
    pub name_id: &'static str,
    pub account_type: AccountType,
    pub parent_code: Option<&'static str>,
    pub placeholder: bool,
    pub description_en: &'static str,
    pub description_id: &'static str,
}

pub const COMPREHENSIVE_ACCOUNT_TEMPLATE: [ComprehensiveAccountDef; 26] = [
    ComprehensiveAccountDef {
        code: "1000",
        name_en: "Assets",
        name_id: "Aset",
        account_type: AccountType::Asset,
        parent_code: None,
        placeholder: true,
        description_en: "Root placeholder account for Assets",
        description_id: "Akun induk placeholder untuk Aset",
    },
    ComprehensiveAccountDef {
        code: "2000",
        name_en: "Liabilities",
        name_id: "Liabilitas",
        account_type: AccountType::Liability,
        parent_code: None,
        placeholder: true,
        description_en: "Root placeholder account for Liabilities",
        description_id: "Akun induk placeholder untuk Liabilitas",
    },
    ComprehensiveAccountDef {
        code: "3000",
        name_en: "Equity",
        name_id: "Ekuitas",
        account_type: AccountType::Equity,
        parent_code: None,
        placeholder: true,
        description_en: "Root placeholder account for Equity",
        description_id: "Akun induk placeholder untuk Ekuitas",
    },
    ComprehensiveAccountDef {
        code: "4000",
        name_en: "Income",
        name_id: "Pendapatan",
        account_type: AccountType::Income,
        parent_code: None,
        placeholder: true,
        description_en: "Root placeholder account for Income",
        description_id: "Akun induk placeholder untuk Pendapatan",
    },
    ComprehensiveAccountDef {
        code: "5000",
        name_en: "Expenses",
        name_id: "Expenses",
        account_type: AccountType::Expense,
        parent_code: None,
        placeholder: true,
        description_en: "Root placeholder account for Expenses",
        description_id: "Akun induk placeholder untuk Pengeluaran",
    },
    ComprehensiveAccountDef {
        code: "1100",
        name_en: "Current Assets",
        name_id: "Aset Lancar",
        account_type: AccountType::Asset,
        parent_code: Some("1000"),
        placeholder: true,
        description_en: "Liquid cash, bank accounts and short-term holdings",
        description_id: "Kas tunai, rekening bank dan saldo likuid",
    },
    ComprehensiveAccountDef {
        code: "2100",
        name_en: "Current Liabilities",
        name_id: "Liabilitas Lancar",
        account_type: AccountType::Liability,
        parent_code: Some("2000"),
        placeholder: true,
        description_en: "Credit cards, payables and short-term liabilities",
        description_id: "Kartu kredit, tagihan dan hutang jangka pendek",
    },
    ComprehensiveAccountDef {
        code: "3100",
        name_en: "Capital & Reserves",
        name_id: "Modal & Cadangan",
        account_type: AccountType::Equity,
        parent_code: Some("3000"),
        placeholder: true,
        description_en: "Initial balances, reserves and retained earnings",
        description_id: "Saldo awal, cadangan modal dan laba ditahan",
    },
    ComprehensiveAccountDef {
        code: "4100",
        name_en: "Operating Income",
        name_id: "Pendapatan Operasional",
        account_type: AccountType::Income,
        parent_code: Some("4000"),
        placeholder: true,
        description_en: "Salary, wages, professional fees and side income",
        description_id: "Gaji utama, upah dan pendapatan operasional",
    },
    ComprehensiveAccountDef {
        code: "5100",
        name_en: "Bills & Utilities",
        name_id: "Tagihan & Utilitas",
        account_type: AccountType::Expense,
        parent_code: Some("5000"),
        placeholder: true,
        description_en: "Recurring bills, electricity, water, and digital services",
        description_id: "Tagihan rutin, utilitas rumah tangga dan layanan digital",
    },
    ComprehensiveAccountDef {
        code: "5200",
        name_en: "Lifestyle & Personal",
        name_id: "Gaya Hidup & Pribadi",
        account_type: AccountType::Expense,
        parent_code: Some("5000"),
        placeholder: true,
        description_en: "Personal care, clothing, education and recreation",
        description_id: "Kebutuhan pribadi, sandang, pendidikan dan hiburan",
    },
    ComprehensiveAccountDef {
        code: "5300",
        name_en: "Transportation",
        name_id: "Transportasi",
        account_type: AccountType::Expense,
        parent_code: Some("5000"),
        placeholder: true,
        description_en: "Vehicle expenses, parking, fuel, and transit",
        description_id: "Biaya kendaraan, parkir, bensin dan transportasi umum",
    },
    ComprehensiveAccountDef {
        code: "1110",
        name_en: "Cash on Wallet",
        name_id: "Dompet Tunai",
        account_type: AccountType::Asset,
        parent_code: Some("1100"),
        placeholder: false,
        description_en: "Physical cash on hand in wallet or home",
        description_id: "Uang tunai fisik di dompet",
    },
    ComprehensiveAccountDef {
        code: "1120",
        name_en: "Checking Account",
        name_id: "Rekening Operasional",
        account_type: AccountType::Asset,
        parent_code: Some("1100"),
        placeholder: false,
        description_en: "Primary daily checking or transactional bank account",
        description_id: "Rekening bank transaksi dan operasional harian",
    },
    ComprehensiveAccountDef {
        code: "1130",
        name_en: "Savings Account",
        name_id: "Rekening Tabungan",
        account_type: AccountType::Asset,
        parent_code: Some("1100"),
        placeholder: false,
        description_en: "Emergency fund and high-yield savings account",
        description_id: "Rekening tabungan dana darurat",
    },
    ComprehensiveAccountDef {
        code: "2110",
        name_en: "Credit Card",
        name_id: "Kartu Kredit",
        account_type: AccountType::Liability,
        parent_code: Some("2100"),
        placeholder: false,
        description_en: "Revolving credit card balance and charges",
        description_id: "Tagihan dan saldo kartu kredit",
    },
    ComprehensiveAccountDef {
        code: "3110",
        name_en: "Opening Balance",
        name_id: "Saldo Awal",
        account_type: AccountType::Equity,
        parent_code: Some("3100"),
        placeholder: false,
        description_en: "Offsetting equity account for opening balance entries",
        description_id: "Akun penyeimbang ekuitas untuk saldo awal",
    },
    ComprehensiveAccountDef {
        code: "3200",
        name_en: "Retained Earnings",
        name_id: "Laba Ditahan",
        account_type: AccountType::Equity,
        parent_code: Some("3100"),
        placeholder: false,
        description_en: "Accumulated net income retained over time",
        description_id: "Akumulasi saldo laba bersih dari periode lalu",
    },
    ComprehensiveAccountDef {
        code: "4110",
        name_en: "Salary",
        name_id: "Gaji Utama",
        account_type: AccountType::Income,
        parent_code: Some("4100"),
        placeholder: false,
        description_en: "Primary monthly employment salary or wages",
        description_id: "Gaji dan upah penghasilan utama",
    },
    ComprehensiveAccountDef {
        code: "4120",
        name_en: "Other Income",
        name_id: "Pendapatan Lain-lain",
        account_type: AccountType::Income,
        parent_code: Some("4100"),
        placeholder: false,
        description_en: "Freelance, cashback, gifts and secondary income",
        description_id: "Pendapatan sampingan, hadiah, dan cashback",
    },
    ComprehensiveAccountDef {
        code: "5110",
        name_en: "Utilities",
        name_id: "Listrik, Air & Gas",
        account_type: AccountType::Expense,
        parent_code: Some("5100"),
        placeholder: false,
        description_en: "Electricity, water, gas, and waste disposal bills",
        description_id: "Tagihan listrik PLN, PDAM air, dan gas",
    },
    ComprehensiveAccountDef {
        code: "5120",
        name_en: "Online Services & Subscriptions",
        name_id: "Langganan Digital",
        account_type: AccountType::Expense,
        parent_code: Some("5100"),
        placeholder: false,
        description_en: "Streaming, software, cloud storage and web subscriptions",
        description_id: "Langganan aplikasi, cloud, internet dan media digital",
    },
    ComprehensiveAccountDef {
        code: "5210",
        name_en: "Clothes",
        name_id: "Pakaian & Busana",
        account_type: AccountType::Expense,
        parent_code: Some("5200"),
        placeholder: false,
        description_en: "Clothing, apparel, footwear and accessories",
        description_id: "Pakaian, sepatu, dan kebutuhan sandang",
    },
    ComprehensiveAccountDef {
        code: "5220",
        name_en: "Education & Books",
        name_id: "Buku & Pendidikan",
        account_type: AccountType::Expense,
        parent_code: Some("5200"),
        placeholder: false,
        description_en: "Books, courses, tutorials and learning resources",
        description_id: "Buku bacaan, kursus, seminar dan edukasi",
    },
    ComprehensiveAccountDef {
        code: "5230",
        name_en: "Hobbies & Recreation",
        name_id: "Hobi & Hiburan",
        account_type: AccountType::Expense,
        parent_code: Some("5200"),
        placeholder: false,
        description_en: "Gaming, sports, hobbies, entertainment and leisure",
        description_id: "Olahraga, permainan, hobi dan rekreasi santai",
    },
    ComprehensiveAccountDef {
        code: "5310",
        name_en: "Parking & Transit",
        name_id: "Parkir & Transportasi",
        account_type: AccountType::Expense,
        parent_code: Some("5300"),
        placeholder: false,
        description_en: "Parking fees, tolls, public transit tickets and fuel",
        description_id: "Tarif parkir, tiket tol, bensin dan tiket transportasi",
    },
];

pub const BUSINESS_ACCOUNT_TEMPLATE: [ComprehensiveAccountDef; 26] = [
    ComprehensiveAccountDef {
        code: "1000",
        name_en: "Assets",
        name_id: "Aset",
        account_type: AccountType::Asset,
        parent_code: None,
        placeholder: true,
        description_en: "Root placeholder account for Assets",
        description_id: "Akun induk placeholder untuk Aset",
    },
    ComprehensiveAccountDef {
        code: "1100",
        name_en: "Current Assets",
        name_id: "Aset Lancar",
        account_type: AccountType::Asset,
        parent_code: Some("1000"),
        placeholder: true,
        description_en: "Liquid cash, bank accounts and receivables",
        description_id: "Kas tunai, rekening bank dan piutang lancar",
    },
    ComprehensiveAccountDef {
        code: "1110",
        name_en: "Cash on Hand",
        name_id: "Kas Tunai & Petty Cash",
        account_type: AccountType::Asset,
        parent_code: Some("1100"),
        placeholder: false,
        description_en: "Physical cash and petty cash fund",
        description_id: "Kas kecil dan uang tunai operasional",
    },
    ComprehensiveAccountDef {
        code: "1120",
        name_en: "Business Bank Account",
        name_id: "Rekening Bank Bisnis",
        account_type: AccountType::Asset,
        parent_code: Some("1100"),
        placeholder: false,
        description_en: "Primary business checking and operational bank account",
        description_id: "Rekening bank transaksi operasional bisnis",
    },
    ComprehensiveAccountDef {
        code: "1140",
        name_en: "Accounts Receivable",
        name_id: "Piutang Usaha (AR)",
        account_type: AccountType::Asset,
        parent_code: Some("1100"),
        placeholder: false,
        description_en: "Unpaid customer invoices and pending billings",
        description_id: "Tagihan invoice klien yang belum dibayar",
    },
    ComprehensiveAccountDef {
        code: "1500",
        name_en: "Fixed Assets",
        name_id: "Aset Tetap & Peralatan",
        account_type: AccountType::Asset,
        parent_code: Some("1000"),
        placeholder: true,
        description_en: "Long-term property, plant and equipment",
        description_id: "Peralatan kantor dan aset jangka panjang",
    },
    ComprehensiveAccountDef {
        code: "1510",
        name_en: "Equipment & Hardware",
        name_id: "Perangkat Kerja & Peralatan",
        account_type: AccountType::Asset,
        parent_code: Some("1500"),
        placeholder: false,
        description_en: "Computers, cameras, machinery and tools",
        description_id: "Komputer, mesin, dan perlengkapan produksi",
    },
    ComprehensiveAccountDef {
        code: "2000",
        name_en: "Liabilities",
        name_id: "Liabilitas",
        account_type: AccountType::Liability,
        parent_code: None,
        placeholder: true,
        description_en: "Root placeholder account for Liabilities",
        description_id: "Akun induk placeholder untuk Liabilitas",
    },
    ComprehensiveAccountDef {
        code: "2100",
        name_en: "Current Liabilities",
        name_id: "Liabilitas Lancar",
        account_type: AccountType::Liability,
        parent_code: Some("2000"),
        placeholder: true,
        description_en: "Vendor bills, credit lines and accrued payables",
        description_id: "Hutang vendor, kartu kredit dan tagihan jangka pendek",
    },
    ComprehensiveAccountDef {
        code: "2110",
        name_en: "Business Credit Card",
        name_id: "Kartu Kredit Bisnis",
        account_type: AccountType::Liability,
        parent_code: Some("2100"),
        placeholder: false,
        description_en: "Revolving corporate credit card balance",
        description_id: "Tagihan kartu kredit operasional usaha",
    },
    ComprehensiveAccountDef {
        code: "2120",
        name_en: "Accounts Payable",
        name_id: "Hutang Usaha (AP)",
        account_type: AccountType::Liability,
        parent_code: Some("2100"),
        placeholder: false,
        description_en: "Outstanding invoices owed to suppliers and vendors",
        description_id: "Hutang tagihan kepada vendor dan supplier",
    },
    ComprehensiveAccountDef {
        code: "2130",
        name_en: "Tax Payable",
        name_id: "Hutang Pajak (PPh/PPN)",
        account_type: AccountType::Liability,
        parent_code: Some("2100"),
        placeholder: false,
        description_en: "Accrued income and value-added tax payable",
        description_id: "Pajak penghasilan dan PPN yang wajib disetor",
    },
    ComprehensiveAccountDef {
        code: "3000",
        name_en: "Equity",
        name_id: "Ekuitas",
        account_type: AccountType::Equity,
        parent_code: None,
        placeholder: true,
        description_en: "Root placeholder account for Equity",
        description_id: "Akun induk placeholder untuk Ekuitas",
    },
    ComprehensiveAccountDef {
        code: "3100",
        name_en: "Owner's Equity & Capital",
        name_id: "Modal Pemilik & Ekuitas",
        account_type: AccountType::Equity,
        parent_code: Some("3000"),
        placeholder: true,
        description_en: "Capital contributions and owner equity accounts",
        description_id: "Modal disetor dan akun kepemilikan usaha",
    },
    ComprehensiveAccountDef {
        code: "3110",
        name_en: "Paid-in Capital",
        name_id: "Modal Disetor",
        account_type: AccountType::Equity,
        parent_code: Some("3100"),
        placeholder: false,
        description_en: "Initial investment and contributed capital",
        description_id: "Modal awal penyertaan pemilik usaha",
    },
    ComprehensiveAccountDef {
        code: "3120",
        name_en: "Owner's Drawings",
        name_id: "Penarikan Pribadi (Prive)",
        account_type: AccountType::Equity,
        parent_code: Some("3100"),
        placeholder: false,
        description_en: "Owner personal distributions and drawdowns",
        description_id: "Pengambilan dana untuk keperluan pribadi pemilik",
    },
    ComprehensiveAccountDef {
        code: "3200",
        name_en: "Retained Earnings",
        name_id: "Laba Ditahan",
        account_type: AccountType::Equity,
        parent_code: Some("3100"),
        placeholder: false,
        description_en: "Cumulative net profit retained in business",
        description_id: "Akumulasi laba bersih dari periode pembukuan lalu",
    },
    ComprehensiveAccountDef {
        code: "4000",
        name_en: "Income",
        name_id: "Pendapatan",
        account_type: AccountType::Income,
        parent_code: None,
        placeholder: true,
        description_en: "Root placeholder account for Income",
        description_id: "Akun induk placeholder untuk Pendapatan",
    },
    ComprehensiveAccountDef {
        code: "4100",
        name_en: "Operating Revenue",
        name_id: "Pendapatan Usaha",
        account_type: AccountType::Income,
        parent_code: Some("4000"),
        placeholder: true,
        description_en: "Main revenue from services, contracts and product sales",
        description_id: "Pendapatan dari jasa, proyek dan penjualan produk",
    },
    ComprehensiveAccountDef {
        code: "4110",
        name_en: "Client Services & Invoices",
        name_id: "Pendapatan Jasa & Proyek",
        account_type: AccountType::Income,
        parent_code: Some("4100"),
        placeholder: false,
        description_en: "Fee-for-service, client retainer, and project billing",
        description_id: "Penghasilan dari tagihan jasa dan kontrak proyek",
    },
    ComprehensiveAccountDef {
        code: "4120",
        name_en: "Product Sales",
        name_id: "Penjualan Produk",
        account_type: AccountType::Income,
        parent_code: Some("4100"),
        placeholder: false,
        description_en: "Revenue from physical or digital goods sold",
        description_id: "Pendapatan penjualan barang atau produk digital",
    },
    ComprehensiveAccountDef {
        code: "5000",
        name_en: "Expenses",
        name_id: "Beban & Biaya Usaha",
        account_type: AccountType::Expense,
        parent_code: None,
        placeholder: true,
        description_en: "Root placeholder account for Expenses",
        description_id: "Akun induk placeholder untuk Pengeluaran Usaha",
    },
    ComprehensiveAccountDef {
        code: "5100",
        name_en: "Direct Project & Cost of Sales",
        name_id: "Beban Pokok Pendapatan (COGS)",
        account_type: AccountType::Expense,
        parent_code: Some("5000"),
        placeholder: true,
        description_en: "Direct expenses related to delivering client projects",
        description_id: "Biaya langsung untuk pengadaan bahan & tenaga proyek",
    },
    ComprehensiveAccountDef {
        code: "5110",
        name_en: "Subcontractors & Freelancers",
        name_id: "Subkontraktor & Tenaga Lepas",
        account_type: AccountType::Expense,
        parent_code: Some("5100"),
        placeholder: false,
        description_en: "Payments to external freelancers, specialists and contractors",
        description_id: "Pembayaran honor tenaga ahli dan pekerja lepas",
    },
    ComprehensiveAccountDef {
        code: "5200",
        name_en: "Operating Expenses & Overhead",
        name_id: "Beban Operasional & Kantor",
        account_type: AccountType::Expense,
        parent_code: Some("5000"),
        placeholder: true,
        description_en: "Day-to-day administrative and operations overhead",
        description_id: "Biaya umum, utilitas dan operasional kantor harian",
    },
    ComprehensiveAccountDef {
        code: "5210",
        name_en: "Software, SaaS & Hosting",
        name_id: "Langganan Software, SaaS & Hosting",
        account_type: AccountType::Expense,
        parent_code: Some("5200"),
        placeholder: false,
        description_en: "Cloud servers, development tools and business SaaS",
        description_id: "Hosting, API, domain, dan aplikasi kerja tim",
    },
];

fn seed_from_template_slice(
    conn: &Connection,
    template: &[ComprehensiveAccountDef],
    language: &str,
    currency: &str,
) -> Result<Vec<Account>, AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    let is_id = language.to_ascii_lowercase().starts_with("id");
    let mut seeded = Vec::new();
    let mut code_to_id: HashMap<String, String> = HashMap::new();

    for existing in repository::list_all(conn)? {
        code_to_id.insert(existing.code.clone(), existing.id.clone());
    }

    for def in template {
        if let Some(existing_id) = code_to_id.get(def.code) {
            code_to_id.insert(def.code.to_string(), existing_id.clone());
            continue;
        }

        let parent_id = def
            .parent_code
            .and_then(|pcode| code_to_id.get(pcode).cloned());

        let name = if is_id { def.name_id } else { def.name_en };
        let description = if is_id {
            def.description_id
        } else {
            def.description_en
        };

        let account = Account {
            id: generate_id(),
            code: def.code.to_string(),
            name: name.to_string(),
            account_type: def.account_type,
            parent_id,
            currency: currency.to_string(),
            placeholder: def.placeholder,
            hidden: false,
            color: None,
            note: None,
            description: Some(description.to_string()),
            interest_rate: None,
            created_at: now,
        };

        repository::insert(conn, &account)?;
        code_to_id.insert(account.code.clone(), account.id.clone());
        seeded.push(account);
    }

    Ok(seeded)
}

pub fn seed_comprehensive_accounts(
    conn: &Connection,
    language: &str,
    currency: &str,
) -> Result<Vec<Account>, AppError> {
    seed_from_template_slice(conn, &COMPREHENSIVE_ACCOUNT_TEMPLATE, language, currency)
}

pub fn seed_account_template(
    conn: &Connection,
    template: &str,
    language: &str,
    currency: &str,
) -> Result<Vec<Account>, AppError> {
    match template.to_ascii_lowercase().as_str() {
        "business" | "freelance" | "agency" => {
            seed_from_template_slice(conn, &BUSINESS_ACCOUNT_TEMPLATE, language, currency)
        }
        "minimal" => {
            let roots: Vec<ComprehensiveAccountDef> = ROOT_PLACEHOLDER_ACCOUNTS
                .iter()
                .map(|r| ComprehensiveAccountDef {
                    code: r.code,
                    name_en: r.name,
                    name_id: match r.code {
                        "1000" => "Aset",
                        "2000" => "Liabilitas",
                        "3000" => "Ekuitas",
                        "4000" => "Pendapatan",
                        "5000" => "Pengeluaran",
                        _ => r.name,
                    },
                    account_type: r.account_type,
                    parent_code: None,
                    placeholder: true,
                    description_en: r.description,
                    description_id: r.description,
                })
                .collect();
            seed_from_template_slice(conn, &roots, language, currency)
        }
        _ => seed_comprehensive_accounts(conn, language, currency),
    }
}

pub fn seed_root_placeholder_accounts(
    conn: &Connection,
    currency: &str,
) -> Result<Vec<Account>, AppError> {
    seed_comprehensive_accounts(conn, "en", currency)
}
