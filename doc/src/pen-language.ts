export const penLanguage = {
  name: "pen",
  patterns: [
    { include: "#comment" },
    { include: "#string" },
    { include: "#number" },
    { include: "#keyword" },
    { include: "#constant" },
    { include: "#type" },
    { include: "#function" },
  ],
  repository: {
    comment: {
      match: "#.*$",
      name: "comment.line.number-sign.pen",
    },
    constant: {
      match: "\\b(?:false|none|true)\\b",
      name: "constant.language.pen",
    },
    function: {
      match: "\\b(?:go|race)\\b",
      name: "support.function.pen",
    },
    keyword: {
      match: "\\b(?:as|else|export|for|foreign|if|import|in|type)\\b",
      name: "keyword.control.pen",
    },
    number: {
      match: "\\b\\d+(?:\\.\\d+)?\\b",
      name: "constant.numeric.pen",
    },
    string: {
      begin: '"',
      end: '"',
      name: "string.quoted.double.pen",
      patterns: [{ match: "\\\\.", name: "constant.character.escape.pen" }],
    },
    type: {
      match: "\\b(?:any|boolean|error|number|string)\\b",
      name: "support.type.pen",
    },
  },
  scopeName: "source.pen",
};
