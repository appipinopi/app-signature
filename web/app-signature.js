/** Dependency-free creator signature for browsers, Electron and native webviews. */
export async function refreshCreator(signature, { fetch: request = globalThis.fetch } = {}) {
  validateSignature(signature);
  try {
    const response = await request('https://raw.githubusercontent.com/appipinopi/app-signature/main/creator.json', { cache: 'no-cache', redirect: 'error', signal: AbortSignal.timeout(5000) });
    if (!response.ok) return signature;
    // Stream and bound the response before parsing untrusted configuration.
    const reader = response.body.getReader();
    const chunks = []; let size = 0;
    while (true) {
      const { value, done } = await reader.read(); if (done) break;
      size += value.byteLength;
      if (size > 16384) { await reader.cancel(); return signature; }
      chunks.push(value);
    }
    const bytes = new Uint8Array(size); let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
    const updated = { ...signature, creator: JSON.parse(new TextDecoder().decode(bytes)) };
    return validateSignature(updated);
  } catch { return signature; }
}
export function validateSignature(signature) {
  if (!signature?.appName?.trim() || !signature?.creator?.name?.trim()) throw new Error('アプリ名・開発者名が必要です。');
  const ids = new Set(['creator', 'works']);
  const appIds = new Set();
  if ((signature.creator.apps ?? []).length > 32) throw new Error('アプリは32件までです。');
  for (const app of signature.creator.apps ?? []) {
    if (!/^[A-Za-z0-9_-]+$/.test(app.id) || appIds.has(app.id) || !app.name?.trim() || typeof app.description !== 'string') throw new Error('アプリ一覧を確認してください。');
    appIds.add(app.id);
    for (const value of [app.url, app.icon]) {
      if (typeof value !== 'string' || /[\u0000-\u001f\u007f]/.test(value)) throw new Error('アプリのURLを確認してください。');
      const url = new URL(value);
      if (url.protocol !== 'https:' || !url.hostname || url.username || url.password) throw new Error('アプリのURLにはHTTPSを使用してください。');
    }
  }
  for (const link of signature.links ?? []) {
    if (!/^[A-Za-z0-9_-]+$/.test(link.id) || ids.has(link.id) || !link.label?.trim()) throw new Error('リンク設定を確認してください。');
    ids.add(link.id);
  }
  for (const value of [signature.creator.url, ...(signature.creator.worksUrl === undefined ? [] : [signature.creator.worksUrl]), ...(signature.links ?? []).map(link => link.url)]) {
    if (typeof value !== 'string' || /[\u0000-\u001f\u007f]/.test(value)) throw new Error('リンクURLを確認してください。');
    const url = new URL(value);
    if (url.protocol !== 'https:' || !url.hostname || url.username || url.password) throw new Error('リンクには認証情報を含まないHTTPS URLを指定してください。');
  }
  return signature;
}

/** onOpenLink receives a registered ID; resolve it in the native backend. */
export function mountSignature(container, { signature, onOpenLink, onError } = {}) {
  validateSignature(signature);
  if (!container?.replaceChildren) throw new Error('表示先がありません。');
  const document = container.ownerDocument;
  const root = document.createElement('section');
  root.className = 'app-signature';
  root.setAttribute('aria-label', `${signature.appName}の開発者と公式リンク`);
  const message = document.createElement('p');
  message.className = 'app-signature__status';
  message.setAttribute('role', 'status');
  message.hidden = true;
  function link(id, label, url, className) {
    const anchor = document.createElement('a');
    anchor.className = className;
    anchor.href = url;
    anchor.target = '_blank';
    anchor.rel = 'noopener noreferrer';
    anchor.dataset.signatureLink = id;
    anchor.textContent = label;
    anchor.setAttribute('aria-label', `${label}（外部ブラウザーで開く）`);
    if (onOpenLink) anchor.addEventListener('click', async event => {
      event.preventDefault();
      if (anchor.getAttribute('aria-disabled') === 'true') return;
      anchor.setAttribute('aria-disabled', 'true');
      message.hidden = true;
      try { await onOpenLink(id); }
      catch (error) {
        message.textContent = 'ブラウザーを開けませんでした。もう一度お試しください。';
        message.hidden = false;
        onError?.(error);
      } finally { anchor.removeAttribute('aria-disabled'); }
    });
    return anchor;
  }
  root.append(link('creator', `Made by ${signature.creator.name} ↗`, signature.creator.url, 'app-signature__creator'));
  const publicLinks = [...(signature.creator.worksUrl ? [{ id: 'works', label: '私が作ったもの', url: signature.creator.worksUrl }] : []), ...(signature.links ?? [])];
  if (publicLinks.length) {
    const links = document.createElement('nav');
    links.className = 'app-signature__links';
    links.setAttribute('aria-label', '公式リンク');
    for (const item of publicLinks) links.append(link(item.id, `${item.label} ↗`, item.url, 'app-signature__link'));
    root.append(links);
  }
  root.append(message);
  container.replaceChildren(root);
  return { destroy() { if (root.parentNode === container) root.remove(); } };
}
