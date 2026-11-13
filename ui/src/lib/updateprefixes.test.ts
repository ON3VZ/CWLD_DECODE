import { describe, expect, it } from 'vitest';
// @ts-expect-error plain JavaScript tool outside the UI project
import { convertCty } from '../../../tools/update_prefixes.mjs';
import { lookupCall, parseTable } from './dxcc';

const SAMPLE = `Belgium:                  14:  27:  EU:   50.76:    -4.77:    -1.0:  ON:
    ON,OO,OP,OQ,OR,OS,OT,=ON4ABC/LH,=OT100A;
Hawaii:                   31:  61:  OC:   21.13:   157.50:    10.0:  KH6:
    AH6,KH6,NH6,WH6(31)[61]<20.5/156.5>,=KH6XYZ;
Turkey, European:         20:  39:  EU:   41.00:   -28.90:    -2.0:  *TA1:
    *TA1,TB1;
European Russia:          16:  29:  EU:   55.00:   -37.60:    -3.0:  UA:
    RA,RK,UA(16)[29];
Spratly Islands:          26:  50:  AS:    9.88:  -114.23:    -8.0:  1S:
    1S,9M0,BM9S;
`;

describe('tools/update_prefixes.mjs', () => {
  it('turns cty.dat records into table lines', () => {
    const lines = convertCty(SAMPLE);
    expect(lines).toEqual([
      'Belgium|EU|50.8|4.8|ON,OO,OP,OQ,OR,OS,OT',
      'Hawaii|OC|21.1|-157.5|AH6,KH6,NH6,WH6',
      'European Russia|EU|55.0|37.6|RA,RK,UA',
      'Spratly Islands|AS|9.9|114.2|1S,9M0,BM9S',
    ]);
  });
  it('keeps exact callsigns on request, and the result can be read by the lookup', () => {
    const lines = convertCty(SAMPLE, { exact: true });
    expect(lines[0]).toContain('=OT100A');
    expect(lines[0]).not.toContain('/');
    const tb = parseTable(lines.join('\n'));
    expect(lookupCall('ON3VZ', tb)?.entity.name).toBe('Belgium');
    expect(lookupCall('KH6ABC', tb)?.entity.name).toBe('Hawaii');
    expect(lookupCall('OT100A', tb)?.entity.name).toBe('Belgium');
  });
});
