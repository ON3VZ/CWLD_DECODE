import type { ClusterNode } from './types';

/** The nodes offered at the start, same list as the core. The first one is Belgian. */
export const DEFAULT_CLUSTER_NODES: ClusterNode[] = [
  { name: 'ON0DXK (Belgium)', host: 'on0dxk.dyndns.org', port: 8000 },
  { name: 'OE9 (Austria)', host: 'cluster.oe9.at', port: 7373 },
  { name: 'S50CLX (Slovenia)', host: 's50clx.infrax.si', port: 41112 },
  { name: 'S50DXS (Slovenia)', host: 's50dxs.s53m.com', port: 8000 },
  { name: 'HamQTH (Czechia)', host: 'hamqth.com', port: 7300 },
  { name: 'GB7DXG (United Kingdom)', host: 'gb7dxg.lebs.org.uk', port: 7373 },
  { name: 'VE7CC (Canada)', host: 've7cc.net', port: 23 },
  { name: 'DXSpots (USA)', host: 'dxspots.com', port: 23 },
];

/** The value of the node choice in the settings: `host:port`. */
export function nodeKey(n: ClusterNode): string {
  return `${n.host.trim()}:${n.port}`;
}
