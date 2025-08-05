import { SSE } from './sse.js';

document.addEventListener('DOMContentLoaded', () => {
    const chatLog = document.getElementById('chat-log');
    const chatForm = document.getElementById('chat-form');
    const chatInput = document.getElementById('chat-input');
    const sendButton = chatForm.querySelector('button');

    const scrollToBottom = () => {
        chatLog.scrollTop = chatLog.scrollHeight;
    };

    /**
     * Renders a single part of a message (e.g., text, function call).
     * @param {object} part - The message part object from the backend.
     * @returns {string} - The HTML string representation of the part.
     */
    const renderPart = (part) => {
        if (!part) return '';

        switch (part.type) {
            case 'text':
                return marked.parse(part.text || '');

            case 'function_call':
                return `
                    <details class="accordion">
                        <summary>Function Call: ${part.name}</summary>
                        <pre><code>${JSON.stringify(part.args, null, 2)}</code></pre>
                    </details>
                `;

            case 'function_response':
                return `
                    <details class="accordion">
                        <summary>Function Response: ${part.name}</summary>
                        <pre><code>${JSON.stringify(part.response, null, 2)}</code></pre>
                    </details>
                `;

            default:
                return `<pre><code>${JSON.stringify(part, null, 2)}</code></pre>`;
        }
    };

    /**
     * Creates and appends a new message bubble to the chat log.
     * @param {object} message - A message object { role, parts }.
     */
    const appendMessage = (message) => {
        const messageDiv = document.createElement('div');
        messageDiv.classList.add('message', message.role);
        messageDiv.dataset.role = message.role;

        const roleDiv = document.createElement('div');
        roleDiv.classList.add('role');
        roleDiv.textContent = message.role;

        const contentDiv = document.createElement('div');
        contentDiv.classList.add('content');

        contentDiv.innerHTML = message.parts.map(renderPart).join('');

        messageDiv.appendChild(roleDiv);
        messageDiv.appendChild(contentDiv);
        chatLog.appendChild(messageDiv);
    };

    /**
     * Checks the last message and either updates it or appends a new one.
     * @param {object} message - The incoming message object.
     */
    const addOrUpdateMessage = (message) => {
        const lastMessageElement = chatLog.lastElementChild;

        // Note: We only merge 'model' and 'tool' roles during live streaming.
        // A user message should always be a new bubble.
        if (lastMessageElement && lastMessageElement.dataset.role === message.role && message.role !== 'user') {
            const contentDiv = lastMessageElement.querySelector('.content');
            const newContentHtml = message.parts.map(renderPart).join('');
            contentDiv.innerHTML += newContentHtml;
        } else {
            appendMessage(message);
        }

        scrollToBottom();
    };

    /**
     * Fetches and displays the initial chat history.
     */
    const loadHistory = async () => {
        try {
            const response = await fetch('/chat');
            if (!response.ok) throw new Error(`HTTP error! Status: ${response.status}`);
            const history = await response.json();
            chatLog.innerHTML = '';

            // --- THIS IS THE FIX ---
            // The server sends a raw log. We need to merge consecutive messages
            // from the same role (except 'user') before rendering.
            const mergedHistory = history.reduce((accumulator, currentMessage) => {
                const lastMessage = accumulator[accumulator.length - 1];

                // Condition for merging: last message exists, roles match, and it's not a user message.
                if (lastMessage && lastMessage.role === currentMessage.role && currentMessage.role !== 'user') {
                    // Merge parts from the current message into the last one.
                    lastMessage.parts.push(...currentMessage.parts);
                } else {
                    // It's a new role, so add it as a new message.
                    // We do a deep copy to avoid modifying the original history array in memory.
                    accumulator.push(JSON.parse(JSON.stringify(currentMessage)));
                }

                return accumulator;
            }, []);

            // Now, render the processed, merged history.
            mergedHistory.forEach(appendMessage);
            // --- END OF FIX ---

        } catch (error) {
            console.error('Failed to load chat history:', error);
            const systemMessage = {
                role: 'system',
                parts: [{ type: 'text', text: `Error loading history: ${error.message}` }]
            };
            appendMessage(systemMessage);
        }
    };

    /**
     * Handles the form submission to send a new message.
     * @param {Event} event
     */
    const handleFormSubmit = (event) => {
        event.preventDefault();
        const inputText = chatInput.value.trim();
        if (!inputText) return;

        const userMessage = {
            role: 'user',
            parts: [{ type: 'text', text: inputText }]
        };

        addOrUpdateMessage(userMessage); // Use this to ensure it's a new bubble
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
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            chatForm.dispatchEvent(new Event('submit'));
        }
    });

    loadHistory();
});