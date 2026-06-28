import fs from 'node:fs'
import { markdownTable }  from 'markdown-table'

async function readData() {
  const data = {};
  const dir = "./target/criterion";

  const groups = await fs.promises.readdir(dir);
  for (const group of groups) {
    data[group] ||= {};

    const benches = await fs.promises.readdir(`${dir}/${group}`);
    for (const bench of benches) {
      data[group][bench] ||= {};

      const measurements = await fs.promises.readdir(`${dir}/${group}/${bench}`);
      for (const measurement of measurements) {
        const json = await import(`${dir}/${group}/${bench}/${measurement}/new/estimates.json`, { with: { type: "json" } });
        const duration_ms = json.default.mean.point_estimate / 1_000_000;
        data[group][bench][measurement] ||= { duration_ms };
      }
    }
  }

  return data
}

async function main() {
  const data = await readData();
  const groups = Object.keys(data);

  for (const group of groups) {
    // Per-group columns/rows: parsers vary by file (tsv is benched on the `.ts`
    // file only), so a single column set taken from the first group would drop
    // tsv or throw on the groups that lack it.
    const columns = Object.keys(data[group]);
    const rows = Object.keys(data[group][columns[0]]);

    console.log(`### ${group}`);
    console.log()
    const table = [["", ...columns]];
    for (const row of rows) {
      const column_numbers = columns.map((column) => data[group][column][row].duration_ms);
      const minimum = Math.min(...column_numbers);
      const column_values = column_numbers.map((number) => {
        return `\`${number.toFixed(1)} ms\` (${(number / minimum).toFixed(2)}x)`
      });
      table.push([row, ...column_values]);
    }
    console.log(markdownTable(table));
    console.log()
  }
}

main()
