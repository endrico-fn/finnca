import { describe, it, expect } from 'vitest';
import { parseCsvStatement, detectBankAdjustment } from './statementParser';

describe('statementParser', () => {
  describe('detectBankAdjustment', () => {
    it('detects BCA admin fees correctly', () => {
      const res = detectBankAdjustment('BIAYA ADM BULANAN', -15000);
      expect(res.isFeeOrInterest).toBe(true);
      expect(res.type).toBe('FEE');
      expect(res.suggestedAccountKeyword).toBe('adm');
    });

    it('detects Mandiri administration charges', () => {
      const res = detectBankAdjustment('BIAYA ADMINISTRASI REKENING', -12500);
      expect(res.isFeeOrInterest).toBe(true);
      expect(res.type).toBe('FEE');
    });

    it('detects BRI debit card fees', () => {
      const res = detectBankAdjustment('BIAYA KARTU ATM', -5000);
      expect(res.isFeeOrInterest).toBe(true);
      expect(res.type).toBe('FEE');
    });

    it('detects interest tax deductions', () => {
      const res = detectBankAdjustment('PAJAK BUNGA TABUNGAN', -3500);
      expect(res.isFeeOrInterest).toBe(true);
      expect(res.type).toBe('FEE');
    });

    it('detects english monthly fees', () => {
      const res = detectBankAdjustment('MONTHLY MAINTENANCE FEE', -25000);
      expect(res.isFeeOrInterest).toBe(true);
      expect(res.type).toBe('FEE');
    });

    it('detects bank interest earnings', () => {
      const res = detectBankAdjustment('BUNGA REKENING TABUNGAN', 45000);
      expect(res.isFeeOrInterest).toBe(true);
      expect(res.type).toBe('INTEREST');
      expect(res.suggestedAccountKeyword).toBe('bunga');
    });

    it('detects jasa giro credit', () => {
      const res = detectBankAdjustment('JASA GIRO BULAN INI', 120000);
      expect(res.isFeeOrInterest).toBe(true);
      expect(res.type).toBe('INTEREST');
    });

    it('returns false for regular grocery and retail expenses', () => {
      const res = detectBankAdjustment('SUPERMARKET GRAND LUCKY', -450000);
      expect(res.isFeeOrInterest).toBe(false);
      expect(res.type).toBeNull();
    });

    it('returns false for payroll and salary credits', () => {
      const res = detectBankAdjustment('PAYROLL GAJI BULAN SEPTEMBER', 15000000);
      expect(res.isFeeOrInterest).toBe(false);
      expect(res.type).toBeNull();
    });

    it('handles empty or undefined descriptions gracefully', () => {
      expect(detectBankAdjustment(undefined, -5000).isFeeOrInterest).toBe(false);
      expect(detectBankAdjustment('', -5000).isFeeOrInterest).toBe(false);
    });
  });

  describe('parseCsvStatement', () => {
    it('parses dual debit/credit column csv correctly', () => {
      const csv = `Date,Description,Debit,Credit\n2026-09-01,Transfer In,,500000\n2026-09-02,BIAYA ADM,15000,`;
      const rows = parseCsvStatement(csv, 'IDR', true);
      expect(rows.length).toBe(2);
      expect(rows[0].date).toBe('2026-09-01');
      expect(rows[0].amount).toBe(500000);
      expect(rows[1].date).toBe('2026-09-02');
      expect(rows[1].amount).toBe(-15000);
    });

    it('parses signed amount column csv correctly', () => {
      const csv = `Tanggal,Keterangan,Nominal\n2026-09-05,Bunga Bank,12500\n2026-09-06,Belanja Mart,-75000`;
      const rows = parseCsvStatement(csv, 'IDR', true);
      expect(rows.length).toBe(2);
      expect(rows[0].amount).toBe(12500);
      expect(rows[1].amount).toBe(-75000);
    });
  });
});
