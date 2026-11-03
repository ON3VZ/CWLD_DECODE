export interface Vars {
  [name: string]: string | undefined;
}

export interface Expanded {
  text: string;
  /** Variables that were used in the template but have no value. */
  missing: string[];
}

/** Replaces `{NAME}` placeholders. Unknown or empty variables are reported, not guessed. */
export function expand(template: string, vars: Vars): Expanded {
  const missing: string[] = [];
  const text = template.replace(/\{([A-Z0-9_]+)\}/g, (_m, name: string) => {
    const v = vars[name];
    if (v === undefined || v.trim() === '') {
      if (!missing.includes(name)) missing.push(name);
      return `{${name}}`;
    }
    return v;
  });
  return { text, missing };
}
