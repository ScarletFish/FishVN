export function neighborsFromSections(sections) {
  const ids = [];
  for (const section of sections) {
    for (const item of section.items) ids.push(item.id);
  }
  return ids;
}
