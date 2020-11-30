// Node script to generate all the unicode ranges for character classes

const unicodeRanges = require('unicode-range-json');
const fs = require('fs');

let output = "";

unicodeRanges.forEach((x) => {
    output += "\npub const " + x['category'].replace(/ /g, "_").replace(/-/g, "_").toUpperCase() + ": (u32, u32) = (" + x['range'][0].toString() + ", " + x['range'][1].toString() + ");";
})

let file = fs.writeFile('unicode_ranges.rs', output, (err) => {});