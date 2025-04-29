export function formatToolParameters(tool) {
  try {
    const json = JSON.parse(tool.parameters);

    const type = tool.type.toLowerCase();
    if (type == 'ahp') {
      return json.params;
    } else if (type == 'mcp') {
      return json.properties;
    }
  } catch {
    return {};
  }
}
