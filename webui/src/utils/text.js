const regexThink = /<think>([\s\S]*?)<\/think>/gi;

export function extractThinkBlock(text) {
  if (typeof text === 'string') {
    const contents = [];

    let match;
    while ((match = regexThink.exec(text)) !== null) {
      contents.push(match[1]);
    }

    return contents.join('\n').trim();
  }

  return '';
}

export function removeThinkBlock(text) {
  if (typeof text === 'string') {
    return text.replace(regexThink, '').trim();
  }

  return text;
}
