import { getPref } from '$lib/core/state/prefs';

export function formatDateToDisplay(dateStr: string, forceFormat?: 'iso' | 'slash'): string {
  if (!dateStr) return '';
  const fmt = forceFormat ?? getPref('finnca_date_format', 'iso');
  if (fmt === 'slash') {
    const parts = dateStr.split('-');
    if (parts.length === 3) {
      return `${parts[2]}/${parts[1]}/${parts[0]}`;
    }
  }
  return dateStr;
}

export function todayString(d: Date = new Date()): string {
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

export function diffCalendarDays(dateA: string, dateB: string): number {
  const [yA, mA, dA] = dateA.split('-').map(Number);
  const [yB, mB, dB] = dateB.split('-').map(Number);
  const utcA = Date.UTC(yA, mA - 1, dA);
  const utcB = Date.UTC(yB, mB - 1, dB);
  return Math.trunc((utcA - utcB) / 86_400_000);
}

export function parseLocalDateParts(value: string): { y: number; m: number; d: number } | null {
  const parts = value.split('-').map(Number);
  if (parts.length !== 3 || parts.some((n) => !Number.isFinite(n))) return null;
  return { y: parts[0], m: parts[1], d: parts[2] };
}

export function daysInMonth(year: number, month1Based: number): number {
  return new Date(year, month1Based, 0).getDate();
}

export function matchesMonthlyDay(
  year: number,
  month1Based: number,
  dayOfMonth: number,
  dayNum: number
): boolean {
  return dayNum === Math.min(dayOfMonth, daysInMonth(year, month1Based));
}

export function endOfMonthString(yearMonth: string): string {
  const [y, m] = yearMonth.split('-').map(Number);
  const lastDay = new Date(y, m, 0).getDate();
  return `${yearMonth}-${String(lastDay).padStart(2, '0')}`;
}

export function shiftMonth(yearMonth: string, delta: number): string {
  const [y, m] = yearMonth.split('-').map(Number);
  const d = new Date(y, m - 1 + delta, 1);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
}

export interface CalendarDay {
  dayNum: number;
  dateStr: string;
  isCurrentMonth: boolean;
}

export function buildCalendarDays(year: number, month: number): CalendarDay[] {
  const firstDay = new Date(year, month, 1);
  const lastDay = new Date(year, month + 1, 0);
  const countDays = lastDay.getDate();
  let startDayOfWeek = firstDay.getDay() - 1;
  if (startDayOfWeek === -1) startDayOfWeek = 6;

  const days: CalendarDay[] = [];
  const prevMonthLastDay = new Date(year, month, 0).getDate();
  for (let i = startDayOfWeek - 1; i >= 0; i--) {
    const d = prevMonthLastDay - i;
    const prevDate = new Date(year, month, -i);
    const dateStr = `${prevDate.getFullYear()}-${String(prevDate.getMonth() + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
    days.push({ dayNum: d, dateStr, isCurrentMonth: false });
  }
  for (let d = 1; d <= countDays; d++) {
    const dateStr = `${year}-${String(month + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
    days.push({ dayNum: d, dateStr, isCurrentMonth: true });
  }
  const totalDays = Math.max(42, Math.ceil(days.length / 7) * 7);
  const remaining = totalDays - days.length;
  for (let d = 1; d <= remaining; d++) {
    const nextDate = new Date(year, month + 1, d);
    const dateStr = `${nextDate.getFullYear()}-${String(nextDate.getMonth() + 1).padStart(2, '0')}-${String(nextDate.getDate()).padStart(2, '0')}`;
    days.push({ dayNum: d, dateStr, isCurrentMonth: false });
  }
  return days;
}
