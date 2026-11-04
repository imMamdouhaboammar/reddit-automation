import { Command } from "commander";
import { spawn } from "node:child_process";
import path from "node:path";
import fs from "node:fs";

export const program = new Command();

program
  .name("reddit-agent")
  .description("Fast, local-first, multi-account Agentic Reddit Operator in Rust")
  .version("0.0.1")
  .allowUnknownOption(true)
  .helpOption("-h, --help", "Display help for command");

export function resolveBinaryPath(): string {
  const rootDir = path.resolve(__dirname, "..");
  const debugBin = path.join(rootDir, "target", "debug", "reddit-agent");
  const releaseBin = path.join(rootDir, "target", "release", "reddit-agent");

  if (fs.existsSync(releaseBin)) {
    return releaseBin;
  }
  if (fs.existsSync(debugBin)) {
    return debugBin;
  }
  return "reddit-agent";
}

export function runCli(args: string[] = process.argv.slice(2)): Promise<number> {
  return new Promise((resolve, reject) => {
    const binPath = resolveBinaryPath();
    const child = spawn(binPath, args, {
      stdio: "inherit",
      env: process.env,
    });

    child.on("error", (err) => {
      reject(err);
    });

    child.on("close", (code) => {
      resolve(code ?? 0);
    });
  });
}

if (import.meta.main) {
  runCli().then((code) => {
    process.exit(code);
  }).catch((err) => {
    console.error("Failed to run reddit-agent:", err.message);
    process.exit(1);
  });
}
