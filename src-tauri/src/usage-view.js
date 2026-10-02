(() => {
  // Return quota scalars only. Never return page text, cookies, names or conversations.
  const dialogs = [...document.querySelectorAll('[role="dialog"], [role="alertdialog"]')];
  let root = dialogs.find(el => /usage limits|usage limit|usage/i.test(el.getAttribute('aria-label') || el.querySelector('h1,h2,h3,[role="heading"]')?.textContent || ''));
  if (!root && /\/settings\/usage\b/.test(location.pathname)) root = document.querySelector('main');
  if (!root) return [];
  const results = [];
  for (const el of root.querySelectorAll('[role="progressbar"]')) {
    const parent = el.parentElement?.parentElement;
    const label = (parent?.textContent || '').slice(0, 1200);
    const value = Number(el.getAttribute('aria-valuenow'));
    const max = Number(el.getAttribute('aria-valuemax') || 100);
    if (!Number.isFinite(value) || max !== 100 || value < 0 || value > 100) continue;
    // Do not guess whether a fill represents used or remaining.
    const orientation = el.getAttribute('aria-label') || label;
    const remaining = /remaining/i.test(orientation);
    const used = /used|utilization/i.test(orientation);
    if (remaining === used || !el.hasAttribute('aria-valuenow')) continue;
    const name = /week/i.test(label) ? 'Weekly' : /5.hour|five.hour/i.test(label) ? '5 hours' : /month/i.test(label) ? 'Monthly budget' : /daily|day/i.test(label) ? 'Daily' : 'Reported window';
    if (!results.some(q=>q.name===name)) results.push({ name, usedPercent: remaining ? 100-value : value });
  }
  // Text-only counters must explicitly state used/remaining and be near a quota label.
  for (const el of root.querySelectorAll('p,span')) {
    if (el.children.length) continue;
    const text = (el.textContent || '').trim();
    const match = text.match(/^(\d+(?:\.\d+)?)\s*%\s*(remaining|used)$/i);
    if (!match) continue;
    const nearby = (el.parentElement?.parentElement?.textContent || '').slice(0,800);
    if (!/week|5.hour|five.hour|month|daily|usage limit/i.test(nearby)) continue;
    const name = /week/i.test(nearby) ? 'Weekly' : /5.hour|five.hour/i.test(nearby) ? '5 hours' : /month/i.test(nearby) ? 'Monthly budget' : 'Reported window';
    const percent = Number(match[1]);
    if (percent>=0 && percent<=100 && !results.some(q=>q.name===name)) results.push({name,usedPercent:match[2].toLowerCase()==='remaining'?100-percent:percent});
  }
  return results.slice(0,12);
})()
