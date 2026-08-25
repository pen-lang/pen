import { readdir } from "node:fs/promises";
import { join, parse } from "node:path";
import sitemap from "@astrojs/sitemap";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import { penLanguage } from "./src/pen-language";

type Item = { slug: string } | { label: string; items: Item[] };

const documentDirectory = "src/content/docs";

const listItems = async (directory: string): Promise<Item[]> =>
  Promise.all(
    (await readdir(join(documentDirectory, directory), { withFileTypes: true }))
      .filter(({ name }) => !name.startsWith("."))
      .sort(
        (one, other) =>
          Number(one.isDirectory()) - Number(other.isDirectory()) ||
          Number(one.name > other.name) - Number(one.name < other.name),
      )
      .map(async (entry) =>
        entry.isDirectory()
          ? {
              items: await listItems(join(directory, entry.name)),
              label: entry.name
                .replaceAll("-", " ")
                .replace(/^./, (character) => character.toUpperCase()),
            }
          : { slug: join(directory, parse(entry.name).name) },
      ),
  );

export default defineConfig({
  integrations: [
    sitemap(),
    starlight({
      customCss: ["./src/index.css"],
      description: "The programming language for scalable development",
      editLink: {
        baseUrl: "https://github.com/pen-lang/pen/edit/main/doc/",
      },
      expressiveCode: {
        shiki: { langs: [penLanguage] },
      },
      favicon: "/icon.svg",
      head: [
        {
          attrs: {
            href: "/manifest.json",
            rel: "manifest",
          },
          tag: "link",
        },
        {
          attrs: {
            "data-domain": "pen-lang.org",
            defer: true,
            src: "https://plausible.io/js/plausible.js",
          },
          tag: "script",
        },
      ],
      logo: {
        src: "./public/icon.svg",
      },
      sidebar: [
        {
          items: [{ label: "Overview", link: "/" }, "roadmap", "the-zen"],
          label: "Home",
        },
        {
          items: [
            "introduction/install",
            "introduction/building-the-first-program",
          ],
          label: "Getting started",
        },
        {
          items: [
            "guides/building-an-executable",
            "guides/creating-a-library",
            "guides/using-a-library",
            "guides/testing",
            "guides/concurrency-and-parallelism",
            "guides/coding-style",
            {
              items: [
                "advanced-features/cross-compile",
                "advanced-features/ffi",
                "advanced-features/writing-system-packages",
              ],
              label: "Advanced features",
            },
          ],
          label: "Guides",
        },
        {
          items: [
            {
              items: [
                "references/language/syntax",
                "references/language/types",
                "references/language/built-ins",
                "references/language/modules",
                "references/language/packages",
              ],
              label: "Language",
            },
            "references/command-line-tools",
            {
              items: await listItems("references/standard-packages"),
              label: "Standard packages",
            },
          ],
          label: "References",
        },
        {
          items: await listItems("examples"),
          label: "Examples",
        },
      ],
      social: [
        {
          href: "https://github.com/pen-lang/pen",
          icon: "github",
          label: "GitHub",
        },
      ],
      title: "Pen",
    }),
  ],
  site: "https://pen-lang.org",
});
