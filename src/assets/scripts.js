// Simple client: fetch CSV and log length
async function fetchCsv() {
  const r = await fetch('/data');
  const reader = r.body.getReader();
  const decoder = new TextDecoder();
  let csv = '';
  for (;;) {
    const { value, done } = await reader.read();
    if (done) break;
    csv += decoder.decode(value, { stream: true });
  }
  console.log('CSV length', csv.length);
}
fetchCsv();