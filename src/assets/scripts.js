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
