import { execSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = __dirname;
const WASM_DIR = join(ROOT_DIR, 'wasm');

function run(command, cwd = ROOT_DIR) {
  console.log(`Running: ${command} (in ${cwd})`);
  try {
    execSync(command, { cwd, stdio: 'inherit' });
    return true;
  } catch (error) {
    return false;
  }
}

console.log("Starting dependency update process...");

// Check for cargo
try {
  execSync('cargo --version', { stdio: 'ignore' });
} catch (e) {
  console.error("Error: cargo is not installed or not in PATH");
  process.exit(1);
}

if (!existsSync(WASM_DIR)) {
  console.error(`Error: WASM folder not found at ${WASM_DIR}`);
  process.exit(1);
}

console.log(`Entering WASM folder at ${WASM_DIR}`);

console.log("Running cargo upgrade...");
// Note: cargo-edit needs to be installed for 'cargo upgrade'
if (!run('cargo upgrade -i allow', WASM_DIR)) {
  console.error("Error: cargo upgrade failed (make sure cargo-edit is installed: cargo install cargo-edit)");
  process.exit(1);
}

console.log("Running cargo update...");
if (run('cargo update', WASM_DIR)) {
  console.log("Dependency update completed successfully");
} else {
  console.error("Error: cargo update failed");
  process.exit(1);
}
