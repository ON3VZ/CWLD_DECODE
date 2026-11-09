import { describe, expect, it } from 'vitest';
import { pickInput, pickPort, silentTest, splitCalls, stationProblems } from './wizard';
import type { InputDevice, PortInfo } from './types';

const port = (name: string, description: string, hint: PortInfo['hint']): PortInfo => ({ name, description, vid: null, pid: null, hint });

describe('pickPort', () => {
  it('takes the only port that looks like the radio', () => {
    expect(pickPort([port('COM3', 'USB Serial', null), port('COM5', 'IC-7300', 'icom')], 'icom')).toBe('COM5');
  });
  it('takes the CI-V port of the two Icom ports', () => {
    const ports = [port('COM6', 'IC-7300MK2 Serial Port B', 'icom'), port('COM5', 'IC-7300MK2 Serial Port A (CI-V)', 'icom')];
    expect(pickPort(ports, 'icom')).toBe('COM5');
  });
  it('chooses nothing when it cannot tell', () => {
    const ports = [port('COM5', 'x', 'icom'), port('COM6', 'y', 'icom')];
    expect(pickPort(ports, 'icom')).toBe('');
    expect(pickPort([], 'qmx')).toBe('');
    expect(pickPort(ports, 'simulator')).toBe('');
  });
});

describe('pickInput', () => {
  const inputs: InputDevice[] = [
    { name: 'Mic', default: true, suggested: false },
    { name: 'Microphone (USB Audio CODEC )', default: false, suggested: true },
  ];
  it('suggests the radio input', () => expect(pickInput(inputs, '')).toBe('Microphone (USB Audio CODEC )'));
  it('keeps a valid choice', () => expect(pickInput(inputs, 'Mic')).toBe('Mic'));
  it('drops an input that is gone', () => expect(pickInput(inputs, 'Old')).toBe('Microphone (USB Audio CODEC )'));
});

describe('stationProblems', () => {
  it('accepts a good station', () => expect(stationProblems({ calls: 'ON3VZ, ON3KC', locator: 'JO20ee' })).toEqual([]));
  it('asks for a call', () => expect(stationProblems({ calls: ' ', locator: '' })[0].key).toBe('wiz.err.nocall'));
  it('names the bad call', () => expect(stationProblems({ calls: 'ON3VZ, XX', locator: '' })[0]).toEqual({ key: 'settings.invalid.call', call: 'XX' }));
  it('rejects a bad locator', () => expect(stationProblems({ calls: 'ON3VZ', locator: 'ZZ99' })[0].key).toBe('settings.invalid.locator'));
});

describe('silentTest', () => {
  it('shows the text and a positive time', () => {
    const r = silentTest('ON3VZ');
    expect(r.text).toBe('CQ CQ DE ON3VZ ON3VZ K');
    expect(r.seconds).toBeGreaterThan(5);
  });
  it('splits calls', () => expect(splitCalls('on3vz, on3kc,')).toEqual(['ON3VZ', 'ON3KC']));
});
