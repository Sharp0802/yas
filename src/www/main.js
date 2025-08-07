import { SSE } from './sse.js';

document.addEventListener('DOMContentLoaded', () => {
    const chatLog = document.getElementById('chat-log');
    const chatForm = document.getElementById('chat-form');
    const chatInput = document.getElementById('chat-input');
    const sendButton = chatForm.querySelector('button');

    const htmlEncode = (input) => {
        return input
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
    }

    const scrollToBottom = () => {
        chatLog.scrollTop = chatLog.scrollHeight;
    };

    const getTextFromParts = (parts) => {
        return parts
            .filter(p => p.type === 'text' && p.text)
            .map(p => p.text)
            .join('');
    };

    const renderPart = (part) => {
        if (!part) return '';
        switch (part.type) {
            case 'text':
                return marked.parse(part.text || '');
            case 'function_call':
                return `
                    <details class="accordion">
                        <summary>Function Call: ${part.name}</summary>
                        <pre><code>${htmlEncode(JSON.stringify(part.args, null, 2))}</code></pre>
                    </details>
                `;
            case 'function_response':
                return `
                    <details class="accordion">
                        <summary>Function Response: ${part.name}</summary>
                        <pre><code>${htmlEncode(JSON.stringify(part.response, null, 2))}</code></pre>
                    </details>
                `;
            default:
                return `<pre><code>${htmlEncode(JSON.stringify(part, null, 2))}</code></pre>`;
        }
    };

    const renderMessageContent = (contentDiv, message) => {
        const textParts = message.parts.filter(p => p.type === 'text');
        const otherParts = message.parts.filter(p => p.type !== 'text');
        let html = '';
        const rawText = getTextFromParts(textParts);
        contentDiv.dataset.rawText = rawText;
        if (rawText) {
            html += marked.parse(rawText);
        }
        html += otherParts.map(renderPart).join('');
        contentDiv.innerHTML = html;
    };

    const appendMessage = (message) => {
        const messageDiv = document.createElement('div');
        messageDiv.classList.add('message', message.role);
        messageDiv.dataset.role = message.role;
        const roleDiv = document.createElement('div');
        roleDiv.classList.add('role');
        roleDiv.textContent = message.role;
        const contentDiv = document.createElement('div');
        contentDiv.classList.add('content');
        renderMessageContent(contentDiv, message);
        messageDiv.appendChild(roleDiv);
        messageDiv.appendChild(contentDiv);
        chatLog.appendChild(messageDiv);
    };

    const addOrUpdateMessage = (message) => {
        const lastMessageElement = chatLog.lastElementChild;
        if (lastMessageElement && lastMessageElement.dataset.role === message.role && message.role !== 'user') {
            const contentDiv = lastMessageElement.querySelector('.content');
            const oldRawText = contentDiv.dataset.rawText || '';
            const newTextChunk = getTextFromParts(message.parts);
            const fullRawText = oldRawText + newTextChunk;
            contentDiv.dataset.rawText = fullRawText;
            const existingNonTextParts = Array.from(contentDiv.querySelectorAll('.accordion'))
                .map(() => ({type: 'non-text-placeholder'}));
            const updatedMessage = {
                parts: [
                    ...existingNonTextParts,
                    ...message.parts.filter(p => p.type !== 'text'),
                    {type: 'text', text: fullRawText}
                ]
            };
            renderMessageContent(contentDiv, updatedMessage);
        } else {
            appendMessage(message);
        }
        scrollToBottom();
    };

    const loadHistory = async () => {
        try {
            const response = await fetch('/chat');
            if (!response.ok) throw new Error(`HTTP error! Status: ${response.status}`);
            const history = await response.json();
            chatLog.innerHTML = '';
            const mergedHistory = history.reduce((accumulator, currentMessage) => {
                const lastMessage = accumulator[accumulator.length - 1];
                if (lastMessage && lastMessage.role === currentMessage.role && currentMessage.role !== 'user') {
                    lastMessage.parts.push(...currentMessage.parts);
                } else {
                    accumulator.push(JSON.parse(JSON.stringify(currentMessage)));
                }
                return accumulator;
            }, []);
            mergedHistory.forEach(appendMessage);
        } catch (error) {
            console.error('Failed to load chat history:', error);
            const systemMessage = {
                role: 'system',
                parts: [{ type: 'text', text: `Error loading history: ${error.message}` }]
            };
            appendMessage(systemMessage);
        }
    };

    const handleFormSubmit = (event) => {
        event.preventDefault();
        const inputText = chatInput.value.trim();
        if (!inputText) return;

        const userMessage = {
            role: 'user',
            parts: [{ type: 'text', text: inputText }]
        };
        addOrUpdateMessage(userMessage);
        scrollToBottom();
        chatInput.value = '';
        chatInput.disabled = true;
        sendButton.disabled = true;

        const sse = new SSE('/chat', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            payload: JSON.stringify(userMessage)
        });

        sse.addEventListener('message', (e) => {
            if (e.data) {
                try {
                    const message = JSON.parse(e.data);
                    addOrUpdateMessage(message);
                } catch (err) {
                    console.error('Failed to parse SSE message data:', e.data, err);
                }
            }
        });

        sse.addEventListener('error', (e) => {
            console.error('SSE Error:', e);
            const errorMessage = {
                role: 'system',
                parts: [{ type: 'text', text: 'Connection error. Please try again.' }]
            };
            appendMessage(errorMessage);
            sse.close();
            chatInput.disabled = false;
            sendButton.disabled = false;
        });

        sse.addEventListener('readystatechange', (e) => {
            if (e.readyState === SSE.CLOSED) {
                console.log('SSE Stream finished and closed.');
                chatInput.disabled = false;
                sendButton.disabled = false;
                chatInput.focus();
            }
        });

        sse.stream();
    };

    chatForm.addEventListener('submit', handleFormSubmit);

    chatInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' && e.altKey) {
            e.preventDefault();
            chatForm.dispatchEvent(new Event('submit', { cancelable: true }));
        }
   });

    loadHistory();
});