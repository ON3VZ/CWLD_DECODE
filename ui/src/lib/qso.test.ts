import { describe, expect, it } from 'vitest';
import { heardIn } from './qso';

describe('heardIn', () => {
  it('takes the sender after DE, not our own call', () => {
    expect(heardIn('ON3VZ DE DL1ABC TNX FER CALL', ['ON3VZ']).call).toBe('DL1ABC');
  });
  it('finds a CQ caller', () => {
    expect(heardIn('CQ CQ DE F4XYZ F4XYZ K', ['ON3VZ']).call).toBe('F4XYZ');
  });
  it('without DE takes the first call that is not ours', () => {
    expect(heardIn('ON3VZ DL1ABC 599', ['ON3VZ']).call).toBe('DL1ABC');
  });
  it('reads name and QTH only after their keyword', () => {
    const h = heardIn('ON3VZ DE DL1ABC UR RST 579 NAME HANS QTH BERLIN HW? K', ['ON3VZ']);
    expect(h.name).toBe('HANS');
    expect(h.qth).toBe('BERLIN');
  });
  it('guesses nothing when keywords are missing', () => {
    const h = heardIn('DL1ABC DE ON3VZ K', ['ON3VZ']);
    expect(h.name).toBeUndefined();
    expect(h.qth).toBeUndefined();
  });
  it('does not take a keyword as name', () => {
    expect(heardIn('DE DL1ABC NAME QTH BERLIN', []).name).toBeUndefined();
  });
  it('gives nothing when only our call is present', () => {
    expect(heardIn('CQ DE ON3VZ K', ['ON3VZ']).call).toBeUndefined();
  });
});
