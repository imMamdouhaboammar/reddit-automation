import { describe, it, expect } from "vitest";
import { program, resolveBinaryPath } from "./index";

describe("TypeScript CLI Wrapper", () => {
  it("initializes the commander program with correct metadata", () => {
    expect(program.name()).toBe("reddit-agent");
    expect(program.version()).toBe("0.0.1");
    expect(program.description()).toContain("Agentic Reddit Operator");
  });

  it("resolves the compiled Rust binary path", () => {
    const binPath = resolveBinaryPath();
    expect(typeof binPath).toBe("string");
    expect(binPath).toContain("reddit-agent");
  });
});
