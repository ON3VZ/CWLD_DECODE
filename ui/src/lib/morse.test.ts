import { describe, expect, it } from 'vitest';
import { durationUnits, durationSeconds } from './morse';

describe('morse timing', () => {
  it('PARIS is 50 units including the trailing word gap', () => {
    expect(durationUnits('PARIS') + 7).toBe(50);
  });
  it('20 WPM PARIS takes 3 seconds with the word gap', () => {
    expect(durationSeconds('PARIS', 20) + 7 * (1.2 / 20)).toBeCloseTo(3.0, 5);
  });
  it('ignores unknown characters', () => {
    expect(durationUnits('E~')).toBe(1);
  });
});
