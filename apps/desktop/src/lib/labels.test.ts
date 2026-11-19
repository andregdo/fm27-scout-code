import { describe, expect, it } from "vitest";
import { sponsorshipKindLabel } from "./labels";

describe("sponsorshipKindLabel", () => {
  it("names the codes confirmed on 27.0.0 and 27.0.1", () => {
    expect(sponsorshipKindLabel(22, "27.0.0+2486695", "en")).toBe(
      "Kit manufacturer",
    );
    expect(sponsorshipKindLabel(19, "27.0.1+2503222", "en")).toBe(
      "Shirt sleeve",
    );
    expect(sponsorshipKindLabel(5, "27.0.1+2503222", "zh-CN")).toBe(
      "个人电视转播合同",
    );
  });

  it("keeps raw codes on unconfirmed builds and unnamed kinds", () => {
    expect(sponsorshipKindLabel(22, "27.1.0+2505117", "en")).toBeNull();
    expect(sponsorshipKindLabel(2, "27.0.0+2486695", "en")).toBeNull();
    expect(sponsorshipKindLabel(18, "27.0.1+2503222", "en")).toBeNull();
  });
});
