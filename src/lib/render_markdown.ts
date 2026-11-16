import DOMPurify from "dompurify";
import { Marked, type Tokens } from "marked";
import marked_katex from "marked-katex-extension";
import "katex/dist/katex.min.css";

function escape_html(text: string): string {
    return text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
}

/** Drop one outer fence when the model wrapped the whole answer. */
function unwrap_outer_fence(source: string): string {
    const match = source.trim().match(/^```[a-zA-Z0-9_-]*\n([\s\S]*?)\n```$/);
    if (!match) return source;
    return match[1];
}

const code_slot = /\u0000CODE(\d+)\u0000/g;
const math_span = /\\\[([\s\S]+?)\\\]|\$\$([\s\S]+?)\$\$|\\\(([\s\S]+?)\\\)/g;
const bare_env = /\\begin\{(align\*?|aligned|equation\*?|gather\*?|eqnarray\*?|multline\*?)\}[\s\S]+?\\end\{\1\}/g;

function protect_code(source: string): { text: string; slots: string[] } {
    const slots: string[] = [];
    const text = source.replace(/```[\s\S]*?```|`[^`\n]+`/g, (block) => {
        const token = `\u0000CODE${slots.length}\u0000`;
        slots.push(block);
        return token;
    });
    return { text, slots };
}

function restore_code(source: string, slots: string[]): string {
    return source.replace(code_slot, (_token, index: string) => slots[Number(index)] ?? "");
}

function flatten_math(body: string): string {
    return body.replace(/[ \t]*\r?\n[ \t]*/g, " ").trim();
}

function display_block(body: string): string {
    const line = flatten_math(body);
    if (!line) return "";
    return `\n\n$$\n${line}\n$$\n\n`;
}

function wrap_bare_envs(text: string): string {
    return text.replace(bare_env, (block) => display_block(block));
}

/**
 * marked-katex only treats display math as a block when the dollars sit on
 * their own lines, and its inline rule stops at a newline. Models often emit
 * `$$formula` / newline / `formula$$`, or `\[...\]`, which then stay as text.
 */
function normalize_math(source: string): string {
    const { text, slots } = protect_code(source.replace(/\r\n/g, "\n"));
    let out = "";
    let cursor = 0;
    for (const match of text.matchAll(math_span)) {
        const index = match.index ?? 0;
        out += wrap_bare_envs(text.slice(cursor, index));
        if (match[1] != null) {
            out += display_block(match[1]);
        } else if (match[2] != null) {
            out += display_block(match[2]);
        } else {
            const line = flatten_math(match[3] ?? "");
            out += line ? `$${line}$` : match[0];
        }
        cursor = index + match[0].length;
    }
    out += wrap_bare_envs(text.slice(cursor));
    return restore_code(out, slots);
}

const parser = new Marked({
    gfm: true,
    breaks: true,
});

parser.use(
    marked_katex({
        throwOnError: false,
        nonStandard: true,
        output: "html",
    }),
);

parser.use({
    renderer: {
        html({ text }: Tokens.HTML | Tokens.Tag) {
            return escape_html(text);
        },
    },
});

export function render_markdown(source: string): string {
    const text = normalize_math(unwrap_outer_fence(source));
    if (!text.trim()) return "";
    const raw = parser.parse(text, { async: false });
    return DOMPurify.sanitize(raw);
}
