export type Locale = "en" | "pt-BR";

const en = {
  eyebrow: "local · 127.0.0.1",
  title: "laya-gateway",
  subtitle:
    "Laya picks the tool; the expensive LLM only fills arguments or replies. Inspired by jev-gateway — open-weight System One.",
  routing: "Routing",
  offlinePrefix: "Gateway offline or unreachable ({error}). Start it with ",
  offlineSuffix: ".",
  requests: "Requests",
  forced: "Forced tools",
  passthrough: "Passthrough",
  layaAvg: "Laya avg ms",
  routed: "Routed",
  none: "No tool",
  layaErrors: "Laya errors",
  recent: "Recent requests",
  when: "When",
  mode: "Mode",
  tool: "Tool",
  conf: "Conf",
  layaMs: "Laya ms",
  tokens: "Tokens",
  reason: "Reason",
  emptyPrefix: "No traffic yet. Point your agent at ",
  emptySuffix: ".",
  updated: "Updated",
  privacy: "prompts and args never appear here · metadata only",
  lang: "Language",
} as const;

const ptBR: { [K in keyof typeof en]: string } = {
  eyebrow: "local · 127.0.0.1",
  title: "laya-gateway",
  subtitle:
    "Laya escolhe a tool; o LLM caro só preenche argumentos ou responde. Inspirado no jev-gateway — System One open-weight.",
  routing: "Routing",
  offlinePrefix: "Gateway offline ou inacessível ({error}). Suba com ",
  offlineSuffix: ".",
  requests: "Requests",
  forced: "Forced tools",
  passthrough: "Passthrough",
  layaAvg: "Laya avg ms",
  routed: "Routed",
  none: "No tool",
  layaErrors: "Laya errors",
  recent: "Requests recentes",
  when: "Quando",
  mode: "Mode",
  tool: "Tool",
  conf: "Conf",
  layaMs: "Laya ms",
  tokens: "Tokens",
  reason: "Motivo",
  emptyPrefix: "Ainda sem tráfego. Aponte o agent para ",
  emptySuffix: ".",
  updated: "Atualizado",
  privacy: "prompts e args nunca aparecem aqui · só metadados",
  lang: "Idioma",
};

export const messages = {
  en,
  "pt-BR": ptBR,
} as const;

export type MessageKey = keyof typeof en;

export function t(
  locale: Locale,
  key: MessageKey,
  vars?: Record<string, string>,
): string {
  let text: string = messages[locale][key] ?? messages.en[key];
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      text = text.replace(`{${k}}`, v);
    }
  }
  return text;
}
