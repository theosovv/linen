import * as fs from 'fs';
import * as path from 'path';

import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const resultsDir = path.resolve(__dirname, '../benchmark-results');
const csvDir = path.resolve(__dirname, '../benchmark-csv');

if (!fs.existsSync(csvDir)) {
  fs.mkdirSync(csvDir, { recursive: true });
}

function createCsv(data, filename) {
  const headers = Object.keys(data[0] || {});
  
  const csvRows = [
    headers.join(','),
    ...data.map(row => 
      headers.map(header => {
        const value = row[header];
        const valueStr = String(value);
        return valueStr.includes(',') ? `"${valueStr}"` : valueStr;
      }).join(',')
    )
  ];
  
  const csvContent = csvRows.join('\n');
  
  fs.writeFileSync(path.join(csvDir, filename), csvContent);
  
  console.log(`CSV saved to benchmark-csv/${filename}`);
}

function createSummaryCsv(allData) {
  const summary = [];
  
  Object.entries(allData).forEach(([category, data]) => {
    data.forEach(benchmark => {
      summary.push({
        Category: category,
        Benchmark: benchmark.name,
        'Operations/sec': benchmark['ops/sec'],
        'Margin': benchmark.margin,
        'Samples': benchmark.samples
      });
    });
  });
  
  const headers = ['Category', 'Benchmark', 'Operations/sec', 'Margin', 'Samples'];
  
  const csvRows = [
    headers.join(','),
    ...summary.map(row => 
      headers.map(header => {
        const value = row[header];
        const valueStr = String(value);
        return valueStr.includes(',') ? `"${valueStr}"` : valueStr;
      }).join(',')
    )
  ];
  
  const csvContent = csvRows.join('\n');
  fs.writeFileSync(path.join(csvDir, 'summary.csv'), csvContent);
  
  console.log(`Summary CSV saved to benchmark-csv/summary.csv`);
}

function generateCsvReports() {
  const resultFiles = fs.readdirSync(resultsDir).filter(file => file.endsWith('.json'));
  
  if (resultFiles.length === 0) {
    console.log('No benchmark results found.');
    return;
  }
  
  const allData = {};
  
  for (const file of resultFiles) {
    const filePath = path.join(resultsDir, file);
    const data = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    
    const category = file.replace('.json', '');
    allData[category] = data;
    
    createCsv(data, `${category}.csv`);
  }
  
  createSummaryCsv(allData);
}

generateCsvReports();
