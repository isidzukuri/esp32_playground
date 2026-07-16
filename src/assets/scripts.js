async function initUi() {
  try {
    const csv = await fetchCsv('/data');
    let csv_items = csvToObjects(csv);
    let data_items = fill_data_points_gape(csv_items, 1);
    updateSensorGauge(data_items.at(-1));
    draw_plots(data_items, "1s");

    const system_stats_csv = await fetchCsv('/system_stats');
    let system_stats_items = csvToObjects(system_stats_csv);
    renderTable(system_stats_items, "system_stats");
  } catch (error) {
    // Always handle potential errors (network issues, parsing failures, etc.)
    console.error("Failed to initialize UI:", error);
  }
}
initUi();


function updateSensorGauge(data) {
  // 1. Find the elements on the page
  const tempElement = document.getElementById('temp');
  const humElement = document.getElementById('hum');

  // 2. Update their text content if the elements exist
  if (tempElement && humElement) {
    tempElement.textContent = Math.ceil(data['temperature']);
    humElement.textContent = Math.ceil(data['humidity']);
  } else {
    console.error("Could not find the temperature or humidity elements.");
  }
}

function csvToObjects(csvString) {
  const lines = csvString.trim().split(/\r?\n/);
  if (lines.length < 2) return [];
  const headers = lines[0].split(',').map(header => header.trim());
  return lines.slice(1).map(line => {
    const values = line.split(',').map(value => value.trim());
    return headers.reduce((obj, header, index) => {
      obj[header] = values[index] !== undefined ? values[index] : null;
      return obj;
    }, {});
  });
}

// Simple client: fetch CSV and log length
async function fetchCsv(endpoint) {
  let r = await fetch(endpoint);
  let reader = r.body.getReader();
  let decoder = new TextDecoder();
  let csv = '';
  for (;;) {
    let { value, done } = await reader.read();
    if (done) break;
    csv += decoder.decode(value, { stream: true });
  }
  console.log('CSV length', csv.length);
  return csv;
}
// fetchCsv();

function fill_data_points_gape(items, interval) {
  if (!items || items.length === 0) return [];

  // 1. Sort items by timestamp to ensure chronological order
  const sortedItems = [...items].sort((a, b) => Number(a.timestamp) - Number(b.timestamp));

  // 2. Create a lookup map for quick access to original data points
  const itemMap = new Map();
  sortedItems.forEach(item => {
    itemMap.set(Number(item.timestamp), item);
  });

  const filledItems = [];
  const startTimestamp = Number(sortedItems[0].timestamp);
  const endTimestamp = Number(sortedItems[sortedItems.length - 1].timestamp);

  // Keep track of the last seen valid data point to carry forward
  let lastKnownItem = sortedItems[0];

  // 3. Loop from start to end, stepping by the interval in seconds
  for (let currentTs = startTimestamp; currentTs <= endTimestamp; currentTs += interval) {
    if (itemMap.has(currentTs)) {
      // If we have an exact match, use it and update our "last known" point
      const currentItem = itemMap.get(currentTs);
      filledItems.push({ ...currentItem });
      lastKnownItem = currentItem;
    } else {
      // Gap detected: Create a new data point using the last known values
      filledItems.push({
        ...lastKnownItem,
        timestamp: String(currentTs) // Update only the timestamp
      });
    }
  }

  return filledItems;
}

function createChatUi() {
  const chatContainer = document.getElementById('chat');
  if (!chatContainer) return;

  const chatLog = document.getElementById('chat-log');
  const chatName = document.getElementById('chat-name');
  const chatMessage = document.getElementById('chat-message');
  const chatSend = document.getElementById('chat-send');

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.host}/ws`;
  const socket = new WebSocket(wsUrl);

  function appendChatLine(text, className = '') {
    if (!chatLog) return;
    const line = document.createElement('div');
    line.className = `chat-line ${className}`.trim();
    line.textContent = text;
    chatLog.appendChild(line);
    chatLog.scrollTop = chatLog.scrollHeight;
  }

  socket.addEventListener('open', () => {
    appendChatLine('Connected to chat', 'chat-status');
  });

  socket.addEventListener('message', (event) => {
    try {
      const data = JSON.parse(event.data);
      appendChatLine(`${data.user_name}: ${data.message}`);
    } catch (err) {
      appendChatLine(event.data, 'chat-raw');
    }
  });

  socket.addEventListener('close', () => {
    appendChatLine('Chat connection closed', 'chat-status');
  });

  socket.addEventListener('error', () => {
    appendChatLine('WebSocket error', 'chat-status');
  });

  function sendMessage() {
    const name = chatName.value.trim() || 'Anonymous';
    const message = chatMessage.value.trim();
    if (!message || socket.readyState !== WebSocket.OPEN) return;

    const payload = JSON.stringify({ user_name: name, message });
    socket.send(payload);
    chatMessage.value = '';
    chatMessage.focus();
  }

  chatSend.addEventListener('click', sendMessage);
  chatMessage.addEventListener('keydown', (event) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      sendMessage();
    }
  });
}

createChatUi();

const SCALES = {
  "1s": 1,
  "1m": 60,
  "10m": 600,
  "1h": 3600,
  "8h": 28800,
  "1d": 86400,
  "1w": 604800,
  "mn": 2592000
};

function draw_plots(data_items, default_scale = "1m") {
  const wrapper = document.getElementById('plots-wrapper');
  const controls = document.getElementById('scale-controls');
  
  wrapper.innerHTML = '';
  controls.innerHTML = '';
  
  if (!data_items || data_items.length === 0) return;
  const keys = Object.keys(data_items[0]).filter(k => k !== 'timestamp');

  // 1. Generate controls styled to match your dark theme
  Object.keys(SCALES).forEach(scale => {
    const btn = document.createElement('button');
    btn.className = `scale-btn ${scale === default_scale ? 'active' : ''}`;
    btn.innerText = scale;
    btn.onclick = () => {
      document.querySelectorAll('.scale-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      renderAll(scale);
    };
    controls.appendChild(btn);
  });

  // 2. Render all charts
  function renderAll(scale) {
    wrapper.innerHTML = ''; 
    
    const step = SCALES[scale];
    const filteredData = [];
    let nextTargetTimestamp = Number(data_items[0].timestamp);
    
    for (let i = 0; i < data_items.length; i++) {
      const currentTs = Number(data_items[i].timestamp);
      if (currentTs >= nextTargetTimestamp) {
        filteredData.push(data_items[i]);
        nextTargetTimestamp = currentTs + step;
      }
    }

    keys.forEach(key => {
      const container = document.createElement('div');
      container.className = 'plot-container';

      const title = document.createElement('h3');
      title.className = 'plot-title';
      title.innerText = key;

      const svgMarkup = generateSVGChart(filteredData, key, step);
      
      container.appendChild(title);
      container.insertAdjacentHTML('beforeend', svgMarkup);
      wrapper.appendChild(container);
    });
  }

  // 3. Mathematical generation of SVG markup 
  function generateSVGChart(data, key, step) {
    const width = 800;
    const height = 220;
    const padLeft = 60;
    const padRight = 20;
    const padTop = 15;
    const padBottom = 35;
    
    const chartWidth = width - padLeft - padRight;
    const chartHeight = height - padTop - padBottom;

    const values = data.map(d => parseFloat(d[key]));
    let minVal = Math.min(...values);
    let maxVal = Math.max(...values);
    
    if (minVal === maxVal) {
      minVal -= 1;
      maxVal += 1;
    }

    const valueRange = maxVal - minVal;

    // Calculate plot coordinates
    const points = data.map((d, i) => {
      const x = padLeft + (i / (data.length - 1 || 1)) * chartWidth;
      const y = padTop + chartHeight - ((parseFloat(d[key]) - minVal) / valueRange) * chartHeight;
      return { x, y };
    });

    const linePath = points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');
    
    let areaPath = '';
    if (points.length > 0) {
      areaPath = `${linePath} L ${points[points.length - 1].x} ${padTop + chartHeight} L ${points[0].x} ${padTop + chartHeight} Z`;
    }

    // Build gridlines and tick marks
    let yGridMarkup = '';
    for (let i = 0; i <= 3; i++) {
      const ratio = i / 3;
      const y = padTop + chartHeight - (ratio * chartHeight);
      const valLabel = (minVal + (ratio * valueRange)).toFixed(2);
      yGridMarkup += `
        <line x1="${padLeft}" y1="${y}" x2="${width - padRight}" y2="${y}" class="grid-line" />
        <text x="${padLeft - 8}" y="${y + 3}" class="axis-text y-label">${valLabel}</text>
      `;
    }

    let xGridMarkup = '';
    const maxLabels = Math.min(data.length, 5);
    for (let i = 0; i < maxLabels; i++) {
      const idx = Math.floor((i / (maxLabels - 1 || 1)) * (data.length - 1));
      if (data[idx]) {
        const x = padLeft + (idx / (data.length - 1 || 1)) * chartWidth;
        const date = new Date(Number(data[idx].timestamp) * 1000);
        
        let timeStr = date.toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit' });
        if (step < 60) timeStr = date.toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
        if (step >= 86400) timeStr = date.toLocaleDateString([], { month: 'short', day: 'numeric' });

        xGridMarkup += `
          <line x1="${x}" y1="${padTop}" x2="${x}" y2="${padTop + chartHeight}" class="grid-line" />
          <text x="${x}" y="${padTop + chartHeight + 18}" class="axis-text x-label">${timeStr}</text>
        `;
      }
    }

    return `
      <svg viewBox="0 0 ${width} ${height}" preserveAspectRatio="xMidYMid meet">
        ${yGridMarkup}
        ${xGridMarkup}
        <line x1="${padLeft}" y1="${padTop}" x2="${padLeft}" y2="${padTop + chartHeight}" class="axis-line" />
        <line x1="${padLeft}" y1="${padTop + chartHeight}" x2="${width - padRight}" y2="${padTop + chartHeight}" class="axis-line" />
        ${areaPath ? `<path d="${areaPath}" class="plot-area" />` : ''}
        ${linePath ? `<path d="${linePath}" class="plot-line" />` : ''}
      </svg>
    `;
  }

  renderAll(default_scale);
}
// draw_plots(data_items, "1s");

function renderTable(data, containerId) {
  const container = document.getElementById(containerId);
  if (!container) return;

  // 1. Separate Global Stats from Task Stats
  let freeHeap = "N/A";
  let minFreeHeap = "N/A";
  const tasksMap = new Map(); // Use a Map to combine 'cpu' and 'time' metrics per task

  data.forEach(item => {
    const metric = item.metric;
    const value = item.value;

    if (metric === 'free_heap') freeHeap = value;
    else if (metric === 'min_free_heap') minFreeHeap = value;
    else if (metric.startsWith('task:')) {
      // Metric format: "task:TaskName:cpu" or "task:TaskName:time"
      const parts = metric.split(':');
      if (parts.length === 3) {
        const taskName = parts[1];
        const type = parts[2]; // 'cpu' or 'time'

        if (!tasksMap.has(taskName)) {
          tasksMap.set(taskName, { name: taskName, time: 'N/A', cpu: 'N/A' });
        }
        
        tasksMap.get(taskName)[type] = value;
      }
    }
  });

  // 2. Build the HTML Output
  let html = `
    <div class="system-status-dashboard">
      <div class="heap-summary">
        <strong>Free Heap:</strong>          ${freeHeap} bytes <br/>
        <strong>Min Ever Free Heap:</strong> ${minFreeHeap} bytes
      </div>

      <table class="task-table">
        <thead>
          <tr>
            <th>Task Name</th>
            <th>Abs Time (Cycles)</th>
            <th>% CPU</th>
          </tr>
        </thead>
        <tbody>
  `;

  // 3. Loop through tasks and populate table rows
  tasksMap.forEach(task => {
    html += `
      <tr>
        <td>${task.name}</td>
        <td>${task.time}</td>
        <td>${task.cpu}</td>
      </tr>
    `;
  });

  html += `
        </tbody>
      </table>
    </div>
  `;

  // 4. Inject into the DOM
  container.innerHTML = html;
}