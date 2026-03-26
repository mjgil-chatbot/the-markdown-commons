# Speed Testing

This crate now has a baseline Criterion harness in `benches/parse.rs` and fixed input files under `benchmarks/corpus/`. The goal of this document is to keep the measurement process repeatable as the shared document model, HTML helpers, cache layer, and `egui` renderer evolve.

The current implementation state described here matches the extraction/consolidation work tracked in chat `ae7be852`.

## What to measure

- Shared document parsing throughput for short, medium, and large documents
- HTML wrapper throughput with and without cache reuse
- Peak slowdown from tables, footnotes, and dense inline formatting
- `egui` rendering cost for wide tables and long code blocks, including repeated cached renders
- Allocation-heavy cases such as repeated reference links and nested emphasis

## Suggested benchmark corpus

Create a small corpus under `benchmarks/corpus/` with:

1. `small.md`: a short README-style file
2. `chat.md`: a long chat transcript with fenced code and lists
3. `tables.md`: wide and narrow tables with inline code
4. `stress.md`: dense links, footnotes, emphasis, HTML tags, and repeated references

## Baseline commands

Run the current harness with:

```bash
cargo bench --release --bench parse
```

If you want a quick external timing pass alongside the Criterion results:

```bash
hyperfine --warmup 3 \
  'cargo test parse_markdown -- --nocapture' \
  'cargo test parse_markdown_fragment -- --nocapture'
```

That is useful for rough iteration, but it is noisy because test startup dominates small workloads.

## Recommended next implementation

1. Keep benchmarking `parse_document` on fixed corpus files because it is now the primary typed API.
2. Keep benchmarking `parse_markdown` and `parse_markdown_fragment` separately as convenience wrapper costs on top of the shared document model.
3. Add direct cached-vs-uncached measurements for `DocumentCache`.
4. Add an `egui` benchmark or profiling harness for `render_markdown` with representative UI widths.
5. Store a baseline result set so regressions are easy to spot across commits.

## Metrics to record

- Median time
- p95 time
- Input size in bytes
- Throughput in MB/s for parser benchmarks
- Relative slowdown versus the previous commit

## Regression policy

- Treat regressions above 10% on stable corpus inputs as worth investigating.
- Do not compare single runs; require repeated samples and a stored baseline.
- Keep the benchmark corpus fixed so changes in numbers reflect code changes, not different inputs.
