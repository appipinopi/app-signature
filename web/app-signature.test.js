import { test } from 'node:test';
import assert from 'node:assert/strict';
import { validateSignature } from './app-signature.js';
const sample = { appName: 'Example', creator: { name: 'appipinopi', url: 'https://github.com/appipinopi' }, links: [{ id: 'website', label: '公式サイト', url: 'https://example.com/' }] };
test('accepts a reusable signature with optional app links', () => {
  assert.equal(validateSignature(sample), sample);
  assert.doesNotThrow(() => validateSignature({ ...sample, links: [] }));
});
test('rejects unsafe URLs and duplicate or reserved IDs', () => {
  for (const url of ['javascript:alert(1)', 'file:///etc/passwd', 'http://example.com', 'https://user:pass@example.com', 'https://example.com\n']) {
    assert.throws(() => validateSignature({ ...sample, creator: { ...sample.creator, url } }));
  }
  assert.throws(() => validateSignature({ ...sample, links: [...sample.links, ...sample.links] }));
  assert.throws(() => validateSignature({ ...sample, links: [{ ...sample.links[0], id: 'creator' }] }));
});
