import fs from "node:fs";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const authorities = [
  "apis/open-api/memory-open-api.openapi.json",
  "apis/app-api/memory-app-api.openapi.json",
  "apis/backend-api/memory-backend-api.openapi.json",
];

for (const relative of authorities) {
  const filePath = path.join(repoRoot, relative);
  const original = fs.readFileSync(filePath, "utf8");
  let updated = original.replace(
    /("name": "page_size"[\s\S]*?"maximum": 200)(\s*\n\s*\})/g,
    (match, head, tail) => (head.includes('"default"') ? match : `${head},\n              "default": 20${tail}`),
  );
  updated = updated.replace(
    /("page_size": \{\s*\n\s*"type": "integer",\s*\n\s*"format": "int32",\s*\n\s*"minimum": 1,\s*\n\s*"maximum": 200)(\s*\n\s*\})/g,
    (match, head, tail) => (head.includes('"default"') ? match : `${head},\n            "default": 20${tail}`),
  );
  if (updated !== original) {
    fs.writeFileSync(filePath, updated);
    console.log(`updated ${relative}`);
  } else {
    console.log(`unchanged ${relative}`);
  }
}
