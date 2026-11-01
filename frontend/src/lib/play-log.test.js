import { describe, expect, it } from 'vitest';
import { averageSeconds, buildCumulativeSeries, monthDay, shortDate } from './play-log.js';

function at(year, month, day, hour = 20) {
  return Math.floor(new Date(year, month - 1, day, hour).getTime() / 1000);
}

function session(startedAt, foreground, alive, ended = true) {
  return {
    started_at: startedAt,
    foreground_seconds: foreground,
    alive_seconds: alive,
    ended_at: ended ? startedAt + alive : null,
  };
}

describe('buildCumulativeSeries', () => {
  it('按开始时刻排序并累加', () => {
    const { points, total } = buildCumulativeSeries([
      session(at(2025, 6, 9), 600, 600),
      session(at(2025, 6, 3), 1200, 1200),
    ]);
    expect(points.map((point) => point.step)).toEqual([1200, 600]);
    expect(points.map((point) => point.total)).toEqual([1200, 1800]);
    expect(total).toBe(1800);
  });

  it('同日多次各占一级台阶', () => {
    const first = at(2025, 6, 8, 10);
    const { points } = buildCumulativeSeries([
      session(first, 600, 600),
      session(first + 3600, 300, 300),
    ]);
    expect(points).toHaveLength(2);
    expect(points[1].total).toBe(900);
  });

  it('按存活口径累计时不混入前景', () => {
    const sessions = [session(at(2025, 6, 8), 600, 3600), session(at(2025, 6, 9), 300, 300)];
    expect(buildCumulativeSeries(sessions).total).toBe(900);
    expect(buildCumulativeSeries(sessions, 'alive').total).toBe(3900);
    expect(buildCumulativeSeries(sessions, 'alive').points[0].step).toBe(3600);
  });

  it('忽略没有开始时刻的行', () => {
    const { points, total } = buildCumulativeSeries([
      session(0, 600, 600),
      { started_at: null, foreground_seconds: 600, alive_seconds: 600 },
      session(at(2025, 6, 8), 600, 600),
    ]);
    expect(points).toHaveLength(1);
    expect(total).toBe(600);
  });

  it('标记进行中的会话并给出首末时刻', () => {
    const start = at(2025, 6, 8);
    const { points, first, last } = buildCumulativeSeries([session(start, 600, 600, false)]);
    expect(points[0].running).toBe(true);
    expect(first).toBe(start);
    expect(last).toBe(start);
  });

  it('没有会话时给出空序列', () => {
    const { points, total, first, last } = buildCumulativeSeries([]);
    expect(points).toEqual([]);
    expect(total).toBe(0);
    expect(first).toBeNull();
    expect(last).toBeNull();
  });
});

describe('averageSeconds', () => {
  it('没有会话时为零', () => {
    expect(averageSeconds(3600, 0)).toBe(0);
  });

  it('按会话数均分', () => {
    expect(averageSeconds(7200, 3)).toBe(2400);
  });
});

describe('shortDate', () => {
  it('按本地日期补零', () => {
    expect(shortDate(at(2025, 3, 5))).toBe('2025-03-05');
  });

  it('空值给空串', () => {
    expect(shortDate(null)).toBe('');
  });
});

describe('monthDay', () => {
  it('只给月日', () => {
    expect(monthDay(at(2025, 3, 5))).toBe('03-05');
  });

  it('空值给空串', () => {
    expect(monthDay(0)).toBe('');
  });
});
