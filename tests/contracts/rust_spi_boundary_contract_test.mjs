import assert from "node:assert/strict";
import fs from "node:fs";

const spiCargo = fs.readFileSync("crates/sdkwork-memory-spi/Cargo.toml", "utf8");
const runtimeCargo = fs.readFileSync("crates/sdkwork-memory-profile-resolver/Cargo.toml", "utf8");

for (const [path, content] of [
  ["crates/sdkwork-memory-spi/Cargo.toml", spiCargo],
  ["crates/sdkwork-memory-profile-resolver/Cargo.toml", runtimeCargo],
]) {
  for (const forbidden of [
    "axum",
    "actix",
    "rocket",
    "hyper",
    "reqwest",
    "sdkwork-memory-sdk",
    "sdkwork-memory-app-sdk",
    "sdkwork-memory-backend-sdk",
  ]) {
    assert.ok(
      !content.includes(forbidden),
      `${path} must remain provider/framework-neutral and must not depend on ${forbidden}`,
    );
  }
}

const spiLib = fs.readFileSync("crates/sdkwork-memory-spi/src/lib.rs", "utf8");
assert.ok(
  !spiLib.includes("async fn"),
  "SPI lib.rs must remain a lightweight module assembly file",
);
