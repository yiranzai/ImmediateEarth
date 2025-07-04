// scripts/updatelog.mjs

import fs from 'fs';
import path from 'path';

const CHANGES = 'CHANGES.md';

export default function updatelog(tag, type = 'updater') {
  const reTag = /## v[\d\.]+/;

  const file = path.join(process.cwd(), CHANGES);

  if (!fs.existsSync(file)) {
    console.log('Could not found CHANGES.md');
    process.exit(1);
  }

  let _tag;
  const tagMap = {};
  const content = fs.readFileSync(file, { encoding: 'utf8' }).split('\n');

  content.forEach((line, index) => {
    if (reTag.test(line)) {
      _tag = line.slice(3).trim();
      if (!tagMap[_tag]) {
        tagMap[_tag] = [];
        return;
      }
    }
    if (_tag) {
      tagMap[_tag].push(line);
    }
    if (reTag.test(content[index + 1])) {
      _tag = null;
    }
  });

  if (!tagMap?.[tag]) {
    console.log(
      `${type === 'release' ? '[CHANGES.md] ' : ''}Tag ${tag} does not exist`
    );
    process.exit(1);
  }

  return tagMap[tag].join('\n').trim() || '';
}