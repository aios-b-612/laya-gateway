import { describe, expect, it } from "vitest";
import { gatewayBaseUrl } from "@/lib/gateway";

describe("gatewayBaseUrl", () => {
  it("defaults to loopback gateway", () => {
    expect(gatewayBaseUrl()).toContain("127.0.0.1");
  });
});
