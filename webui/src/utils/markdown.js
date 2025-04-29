import katex from '@vscode/markdown-it-katex';
import MarkdownIt from 'markdown-it';

const md = new MarkdownIt({
  breaks: true,
});
md.use(katex, {
  displayMode: true,
});

export function renderMarkdown(source) {
  return md.render(source);
}
