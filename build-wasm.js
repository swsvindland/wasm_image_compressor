import { execSync } from 'node:child_process';
import { copyFileSync, readFileSync, writeFileSync, unlinkSync, renameSync, mkdirSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = __dirname;
const WASM_DIR = join(ROOT_DIR, 'wasm');
const PKG_DIR = join(WASM_DIR, 'pkg');
const LIB_DIR = join(PKG_DIR, 'lib');

function run(command, cwd = ROOT_DIR) {
  console.log(`Running: ${command} (in ${cwd})`);
  execSync(command, { cwd, stdio: 'inherit' });
}

try {
  // Build WebAssembly package
  run('wasm-pack build --out-name index --target web --out-dir pkg/lib', WASM_DIR);

  // Copy and move files
  const gitignorePath = join(LIB_DIR, '.gitignore');
  if (existsSync(gitignorePath)) {
    unlinkSync(gitignorePath);
  }

  copyFileSync(join(ROOT_DIR, 'README.md'), join(PKG_DIR, 'README.md'));
  renameSync(join(LIB_DIR, 'package.json'), join(PKG_DIR, 'package.json'));

  // Update package.json
  const pkgPath = join(PKG_DIR, 'package.json');
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));

  pkg.files = ["lib/"];
  pkg.main = "lib/index.js";
  pkg.types = "lib/index.d.ts";

  writeFileSync(pkgPath, JSON.stringify(pkg, null, 2));

  console.log('Build complete!');
} catch (error) {
  console.error('Build failed:', error.message);
  process.exit(1);
}
