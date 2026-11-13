import { describe, expect, it } from 'vitest';
import { PREFIX_TABLE } from './dxcc-data';
import { bearingDeg, describeCall, distanceKm, expandToken, formatKm, locatorToLatLon, lookupCall, parseTable } from './dxcc';

const country = (c: string) => lookupCall(c)?.entity.name ?? null;

describe('prefix table', () => {
  it('every line parses', () => {
    expect(() => parseTable(PREFIX_TABLE)).not.toThrow();
  });
  it('no prefix belongs to two countries', () => {
    const seen = new Map<string, string>();
    const clashes: string[] = [];
    for (const line of PREFIX_TABLE.split('\n')) {
      const [name, , , , list] = line.split('|');
      for (const tok of list.split(',')) {
        if (tok.startsWith('=')) continue;
        for (const p of expandToken(tok)) {
          const other = seen.get(p);
          if (other && other !== name) clashes.push(`${p}: ${other} and ${name}`);
          seen.set(p, name);
        }
      }
    }
    expect(clashes).toEqual([]);
  });
  it('has the continents we expect and no dashes', () => {
    for (const line of PREFIX_TABLE.split('\n')) {
      expect(['EU', 'AS', 'AF', 'NA', 'SA', 'OC']).toContain(line.split('|')[1]);
      expect(/[–—]/.test(line)).toBe(false);
    }
  });
  it('expands letter ranges, lists and wildcards', () => {
    expect(expandToken('D[A-C]')).toEqual(['DA', 'DB', 'DC']);
    expect(expandToken('R[890]')).toEqual(['R8', 'R9', 'R0']);
    expect(expandToken('X?').length).toBe(26);
    expect(expandToken('ON')).toEqual(['ON']);
  });
});

describe('country of a callsign', () => {
  it('finds the usual ones', () => {
    expect(country('ON3VZ')).toBe('Belgium');
    expect(country('OO7ABC')).toBe('Belgium');
    expect(country('DL4SAV')).toBe('Germany');
    expect(country('PA3XYZ')).toBe('Netherlands');
    expect(country('F6ABC')).toBe('France');
    expect(country('W1AW')).toBe('United States');
    expect(country('K1ABC')).toBe('United States');
    expect(country('AA1AA')).toBe('United States');
    expect(country('JA1ABC')).toBe('Japan');
    expect(country('VE3XYZ')).toBe('Canada');
    expect(country('VK2ABC')).toBe('Australia');
    expect(country('ZS6ABC')).toBe('South Africa');
    expect(country('LU1ABC')).toBe('Argentina');
    expect(country('PY2ABC')).toBe('Brazil');
    expect(country('9A1A')).toBe('Croatia');
    expect(country('5B4ABC')).toBe('Cyprus');
    expect(country('4X6ABC')).toBe('Israel');
    expect(country('OZ1ABC')).toBe('Denmark');
    expect(country('OH2ABC')).toBe('Finland');
    expect(country('SM5ABC')).toBe('Sweden');
    expect(country('SP9ABC')).toBe('Poland');
    expect(country('SU1ABC')).toBe('Egypt');
    expect(country('ST2ABC')).toBe('Sudan');
    expect(country('7S2ABC')).toBe('Sweden');
    expect(country('7X2ABC')).toBe('Algeria');
  });
  it('longest prefix wins', () => {
    expect(country('KH6ABC')).toBe('Hawaii');
    expect(country('KL7ABC')).toBe('Alaska');
    expect(country('KP4ABC')).toBe('Puerto Rico');
    expect(country('VK9LM')).toBe('Lord Howe Island');
    expect(country('EA8ABC')).toBe('Canary Islands');
    expect(country('EA6ABC')).toBe('Balearic Islands');
    expect(country('EA3ABC')).toBe('Spain');
    expect(country('IS0ABC')).toBe('Sardinia');
    expect(country('IK2ABC')).toBe('Italy');
    expect(country('GM3ABC')).toBe('Scotland');
    expect(country('GW4ABC')).toBe('Wales');
    expect(country('G4ABC')).toBe('England');
    expect(country('M0ABC')).toBe('England');
    expect(country('2E0ABC')).toBe('England');
    expect(country('OJ0ABC')).toBe('Market Reef');
    expect(country('OH0ABC')).toBe('Aland Islands');
    expect(country('HB0ABC')).toBe('Liechtenstein');
    expect(country('3D2ABC')).toBe('Fiji');
    expect(country('3DA0AB')).toBe('Eswatini');
  });
  it('Russia is split by call area', () => {
    expect(country('UA3ABC')).toBe('European Russia');
    expect(country('RA1ABC')).toBe('European Russia');
    expect(country('RK9ABC')).toBe('Asiatic Russia');
    expect(country('UA0ABC')).toBe('Asiatic Russia');
    expect(country('R9ABC')).toBe('Asiatic Russia');
    expect(country('UA2ABC')).toBe('Kaliningrad');
    expect(country('UR5ABC')).toBe('Ukraine');
    expect(country('UN7ABC')).toBe('Kazakhstan');
  });
  it('portable and visiting calls', () => {
    expect(country('ON3VZ/P')).toBe('Belgium');
    expect(country('ON3VZ/M')).toBe('Belgium');
    expect(country('ON3VZ/QRP')).toBe('Belgium');
    expect(country('W1AW/6')).toBe('United States');
    expect(country('EA8/ON3VZ')).toBe('Canary Islands');
    expect(country('ON3VZ/EA8')).toBe('Canary Islands');
    expect(country('ON3VZ/KH6')).toBe('Hawaii');
    expect(country('DL/ON3VZ/P')).toBe('Germany');
    expect(country('ON3VZ/MM')).toBeNull();
    expect(country('ON3VZ/AM')).toBeNull();
  });
  it('unknown and odd input', () => {
    expect(country('')).toBeNull();
    expect(country('??')).toBeNull();
    expect(country('QQ1ABC')).toBeNull();
  });
});

describe('locators, bearing and distance', () => {
  it('middle of a locator', () => {
    const a = locatorToLatLon('JO21')!;
    expect(a.lat).toBeCloseTo(51.5, 3);
    expect(a.lon).toBeCloseTo(5, 3);
    const b = locatorToLatLon('jo21ee')!;
    expect(b.lat).toBeCloseTo(51.0 + 4 / 24 + 1 / 48 + 0.5 - 0.5 + 0.0, 1);
    expect(b.lon).toBeCloseTo(4 + 4 * (2 / 24) + 1 / 24, 3);
    expect(locatorToLatLon('XX00')).toBeNull();
    expect(locatorToLatLon('JO2')).toBeNull();
    expect(locatorToLatLon('')).toBeNull();
  });
  it('known distances', () => {
    const jo = locatorToLatLon('JO21')!;
    const fn = locatorToLatLon('FN31')!;
    const km = distanceKm(jo, fn);
    expect(km).toBeGreaterThan(5700);
    expect(km).toBeLessThan(6100);
    // Antwerp area to New York area: northwest.
    const b = bearingDeg(jo, fn);
    expect(b).toBeGreaterThan(285);
    expect(b).toBeLessThan(305);
    // Equator, 90 degrees of longitude is a quarter of the circumference.
    expect(distanceKm({ lat: 0, lon: 0 }, { lat: 0, lon: 90 })).toBeCloseTo(10007.5, 0);
    expect(bearingDeg({ lat: 0, lon: 0 }, { lat: 0, lon: 90 })).toBeCloseTo(90, 5);
    expect(bearingDeg({ lat: 0, lon: 0 }, { lat: 10, lon: 0 })).toBeCloseTo(0, 5);
  });
  it('describes a call from the own locator', () => {
    const us = describeCall('W1AW', 'JO21EE', '', 'ON3VZ')!;
    expect(us.country).toBe('United States');
    expect(us.basis).toBe('country');
    expect(us.km!).toBeGreaterThan(6500);
    expect(us.km!).toBeLessThan(9000);
    expect(us.longPath).toBe(Math.round((us.bearing! + 180) % 360) % 360);
    const jp = describeCall('JA1ABC', 'JO21EE', '', 'ON3VZ')!;
    expect(jp.bearing!).toBeGreaterThan(20);
    expect(jp.bearing!).toBeLessThan(50);
  });
  it('a locator from QRZ.com beats the middle of the country', () => {
    const near = describeCall('DL4SAV', 'JO21EE', 'JO31', 'ON3VZ')!;
    expect(near.basis).toBe('grid');
    expect(near.km!).toBeLessThan(250);
    expect(near.home).toBe(false);
  });
  it('own country without a locator of theirs gives no distance', () => {
    const be = describeCall('ON4ABC', 'JO21EE', '', 'ON3VZ')!;
    expect(be.home).toBe(true);
    expect(be.km).toBeNull();
    expect(be.basis).toBeNull();
  });
  it('without own locator only the country', () => {
    const d = describeCall('W1AW', '', '', 'ON3VZ')!;
    expect(d.country).toBe('United States');
    expect(d.km).toBeNull();
    expect(d.bearing).toBeNull();
  });
  it('formats km', () => {
    expect(formatKm(5140)).toBe('5 140');
    expect(formatKm(950)).toBe('950');
    expect(formatKm(12345)).toBe('12 345');
  });
});
