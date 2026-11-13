import { describe, expect, it } from 'vitest';
import { LABEL_DOT_PX, LABEL_GAP_PX, LABEL_MAX_SHIFT_PX, labelWidth, placeLabels } from './scopelabels';

const overlaps = (list: { row: number; left: number }[], widths: number[]) => {
  for (let i = 0; i < list.length; i++) {
    for (let j = i + 1; j < list.length; j++) {
      if (list[i].row !== list[j].row) continue;
      const [a, b] = [list[i].left, list[i].left + widths[i]];
      const [c, d] = [list[j].left, list[j].left + widths[j]];
      if (a < d + LABEL_GAP_PX && c < b + LABEL_GAP_PX) return true;
    }
  }
  return false;
};

describe('labels above the band scope', () => {
  it('puts a lone label right above its frequency', () => {
    const { placed, hidden } = placeLabels([{ anchor: 300, width: 60 }], 800, 3);
    expect(hidden).toBe(0);
    expect(placed[0]).toMatchObject({ row: 0, left: 300 - LABEL_DOT_PX, attach: 300, anchor: 300 });
  });

  it('moves a neighbour aside or down instead of hiding it, and never lets two overlap', () => {
    const wishes = [300, 310, 320, 330, 340, 350].map((anchor) => ({ anchor, width: 60 }));
    const { placed, hidden } = placeLabels(wishes, 800, 3);
    expect(hidden).toBe(0);
    expect(placed).toHaveLength(6);
    expect(overlaps(placed, placed.map(() => 60))).toBe(false);
    for (const p of placed) expect(Math.abs(p.left - (p.anchor - LABEL_DOT_PX))).toBeLessThanOrEqual(LABEL_MAX_SHIFT_PX);
    // The first wish keeps the best place.
    expect(placed[0]).toMatchObject({ row: 0, left: 292 });
  });

  it('keeps the leader on the label and pointing at the frequency', () => {
    const { placed } = placeLabels([{ anchor: 300, width: 60 }, { anchor: 302, width: 60 }], 800, 1);
    const moved = placed[1];
    expect(moved.left).not.toBe(302 - LABEL_DOT_PX);
    expect(moved.anchor).toBe(302);
    expect(moved.attach).toBeGreaterThanOrEqual(moved.left + 5);
    expect(moved.attach).toBeLessThanOrEqual(moved.left + 60 - 5);
  });

  it('keeps labels at the edges inside the scope', () => {
    const { placed } = placeLabels([{ anchor: 2, width: 60 }, { anchor: 798, width: 60 }], 800, 2);
    expect(placed[0].left).toBe(0);
    expect(placed[1].left).toBe(740);
    expect(placed[1].attach).toBe(795);
  });

  it('counts what really does not fit, and what lies outside the scope', () => {
    const crowd = Array.from({ length: 12 }, () => ({ anchor: 400, width: 70 }));
    const { placed, hidden } = placeLabels(crowd, 800, 2);
    expect(placed.length + hidden).toBe(12);
    expect(hidden).toBeGreaterThan(0);
    expect(overlaps(placed, placed.map(() => 70))).toBe(false);
    expect(placeLabels([{ anchor: -5, width: 60 }, { anchor: 900, width: 60 }], 800, 2)).toEqual({ placed: [], hidden: 2 });
  });

  it('makes room for a split badge', () => {
    expect(labelWidth('3B8XX', 'UP 1')).toBeGreaterThan(labelWidth('3B8XX'));
    expect(labelWidth('PA3ABC/P')).toBeGreaterThan(labelWidth('PA3ABC'));
  });
});
