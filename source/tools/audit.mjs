import { readdir, readFile, writeFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';

// Scan all active files and UTF-16 strings. Never follow links.
const [source, installed, forbidden, reportPath, buildPath] = process.argv.slice(2);
if (![source, installed, forbidden, reportPath].every(Boolean)) throw new Error('Usage: audit.mjs source installed forbidden-token report.json');
const needle = forbidden.toLowerCase();
const hits = [];
let files = 0;
async function walk(root) {
  for (const entry of await readdir(root, {withFileTypes:true})) {
    const full = path.join(root, entry.name);
    if (entry.isSymbolicLink()) throw new Error(`Unexpected link: ${full}`);
    if (full.toLowerCase().includes(needle)) hits.push({path:full,kind:'path'});
    if (entry.isDirectory()) await walk(full);
    else if (entry.isFile()) {
      files++;
      const bytes = await readFile(full);
      if (bytes.toString('utf8').toLowerCase().includes(needle) || bytes.toString('utf16le').toLowerCase().includes(needle)) hits.push({path:full,kind:'content'});
    }
  }
}
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
await walk(source); await walk(installed);
const build = await readFile(buildPath || path.resolve(source,'../../BuildCache/ScrollPilot/release/scrollpilot.exe'));
const app = await readFile(path.join(installed,'ScrollMage.exe'));
const mismatches = [];
let sourceFiles = 0;
async function compare(relative='') {
  for (const entry of await readdir(path.join(source,relative),{withFileTypes:true})) {
    if (!relative && entry.name === 'target') continue;
    const next = path.join(relative,entry.name);
    if (entry.isDirectory()) await compare(next);
    else if (entry.isFile()) {
      sourceFiles++;
      try { if (sha(await readFile(path.join(source,next))) !== sha(await readFile(path.join(installed,'source',next)))) mismatches.push(next); }
      catch { mismatches.push(next); }
    }
  }
}
await compare();
const report = {generated_at:new Date().toISOString(),files_scanned:files,forbidden_name_matches:hits,executable_bytes:(await stat(path.join(installed,'ScrollMage.exe'))).size,release_sha256:sha(build),installed_sha256:sha(app),executable_matches:sha(build)===sha(app),source_files_compared:sourceFiles,source_mismatches:mismatches};
await writeFile(reportPath,JSON.stringify(report,null,2)+'\n');
process.stdout.write(JSON.stringify(report,null,2)+'\n');
if (hits.length || mismatches.length || !report.executable_matches) process.exitCode=1;
