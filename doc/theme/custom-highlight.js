hljs.registerLanguage("pen", (hljs) => ({
  name: "Pen",
  keywords: {
    keyword: "as else foreign if import type",
    built_in: "any boolean error none number string",
    literal: "false none true",
  },
  contains: [
    hljs.QUOTE_STRING_MODE,
    hljs.C_NUMBER_MODE,
    {
      scope: "string",
      begin: '"',
      end: '"',
      contains: [{ begin: "\\\\." }],
    },
    hljs.COMMENT("#", "$"),
  ],
}));

hljs.initHighlightingOnLoad();
