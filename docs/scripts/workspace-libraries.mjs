// SPDX-License-Identifier: GPL-2.0-or-later
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// Cargo owns the list: adding/removing a library requires no handbook edits.
const metadata = JSON.parse(execFileSync('cargo', [
  'metadata', '--format-version', '1', '--no-deps', '--locked',
], { cwd: fileURLToPath(new URL('../../', import.meta.url)), encoding: 'utf8' }));

/** @type {{ name: string, href: string }[]} */
export const workspaceLibraries = metadata.packages
  .filter(pkg => metadata.workspace_members.includes(pkg.id))
  .flatMap(pkg => pkg.targets
    .filter(target => target.kind.includes('lib') && target.doc)
    .map(target => ({ name: pkg.name, href: `/razers/api/${target.name.replaceAll('-', '_')}/index.html` })))
  .sort((a, b) => a.name.localeCompare(b.name));
