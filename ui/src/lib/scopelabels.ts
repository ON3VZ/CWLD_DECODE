/**
 * Where the spot labels go above the band scope. Every label wants to sit right above its
 * frequency. When that place is taken it moves a little to the side or a row down, and a thin
 * leader line still points at the exact frequency. Labels that find no place are counted.
 */

export interface LabelWish {
  /** The x of the frequency in pixels from the left edge of the scope. */
  anchor: number;
  /** The width the label needs in pixels. */
  width: number;
}

export interface LabelPlace {
  /** Index in the list of wishes. */
  index: number;
  row: number;
  /** Left edge of the label in pixels. */
  left: number;
  /** Where the leader line leaves the label (bottom edge), in pixels. */
  attach: number;
  anchor: number;
}

/** The dot at the start of a label sits this far from its left edge: it marks the frequency. */
export const LABEL_DOT_PX = 8;
/** Free space kept between two labels in a row. */
export const LABEL_GAP_PX = 4;
/** A label never moves further from its frequency than this. */
export const LABEL_MAX_SHIFT_PX = 64;
/** One row down counts as much as this many pixels to the side. */
const ROW_COST_PX = 5;

/** An estimate of the width of a label: the dot, the callsign and an optional badge. */
export function labelWidth(call: string, badge = ''): number {
  return Math.round(22 + call.length * 6.9 + (badge ? 9 + badge.length * 6.1 : 0));
}

/**
 * Places the labels, the first wish first (so give the most wanted stations first).
 * `areaW` is the width of the scope in pixels, `rowsMax` the number of rows.
 */
export function placeLabels(wishes: LabelWish[], areaW: number, rowsMax: number, maxShift = LABEL_MAX_SHIFT_PX): { placed: LabelPlace[]; hidden: number } {
  const rows: Array<Array<[number, number]>> = Array.from({ length: Math.max(1, rowsMax) }, () => []);
  const placed: LabelPlace[] = [];
  let hidden = 0;
  wishes.forEach((w, index) => {
    if (w.anchor < 0 || w.anchor > areaW) {
      hidden += 1;
      return;
    }
    const max = Math.max(0, areaW - w.width);
    const ideal = w.anchor - LABEL_DOT_PX;
    let best: { row: number; left: number; cost: number } | null = null;
    rows.forEach((taken, row) => {
      const free = (left: number) => left >= 0 && left <= max && taken.every(([a, b]) => left + w.width + LABEL_GAP_PX <= a || left >= b + LABEL_GAP_PX);
      const tries = [Math.min(max, Math.max(0, ideal))];
      for (const [a, b] of taken) tries.push(b + LABEL_GAP_PX, a - LABEL_GAP_PX - w.width);
      for (const left of tries) {
        const shift = Math.abs(left - ideal);
        if (shift > maxShift || !free(left)) continue;
        const cost = shift + row * ROW_COST_PX;
        if (!best || cost < best.cost) best = { row, left, cost };
      }
    });
    if (!best) {
      hidden += 1;
      return;
    }
    const b = best as { row: number; left: number };
    rows[b.row].push([b.left, b.left + w.width]);
    const attach = Math.min(b.left + w.width - 5, Math.max(b.left + 5, w.anchor));
    placed.push({ index, row: b.row, left: b.left, attach, anchor: w.anchor });
  });
  return { placed, hidden };
}
