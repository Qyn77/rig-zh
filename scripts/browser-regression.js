// Run only with an already-authorized browser Tab exposing the documented Playwright API.
// Regression: populated search must close on the first Escape and restore trigger focus.
async function verifySearchEscape(tab) {
  await tab.playwright.getByRole('button', { name: '搜索文档', exact: true }).click();
  await tab.playwright.getByRole('searchbox').fill('工具');
  await tab.playwright.getByRole('searchbox').press('Escape');
  const state = await tab.playwright.evaluate(() => ({
    open: document.querySelector('dialog').open,
    focus: document.activeElement.id,
  }));
  if (state.open || state.focus !== 'search-open') {
    throw new Error(`Search Escape regression: ${JSON.stringify(state)}`);
  }
  return state;
}
