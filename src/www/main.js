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
     * Helper function to extract only the text content from a parts array.
     * @param {Array} parts - The array of parts from a message.
     * @returns {string} - The concatenated text.
     */
    const getTextFromParts = (parts) => {
        return parts
            .filter(p => p.type === 'text' && p.text)
            .map(p => p.text)
            .join('');
    };

    /**
     * Renders a single part of a message (e.g., text, function call).
     * @param {object} part - The message part object from the backend.
     * @returns {string} - The HTML string representation of the part.
     */
    const renderPart = (part) => {
        if (!part) return '';

        // This function now only needs to handle non-text parts for rendering,
        // as text is handled separately. We keep it for rendering history
        // and for when a non-text part appears.
        switch (part.type) {
            case 'text':
                // Text rendering is now handled by the update functions directly.
                // This case is primarily for initial history rendering.
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
     * Renders an entire message object into its content div.
     * @param {HTMLElement} contentDiv - The div to render into.
     * @param {object} message - The message object with its parts array.
     */
    const renderMessageContent = (contentDiv, message) => {
        // Separate text parts from other parts
        const textParts = message.parts.filter(p => p.type === 'text');
        const otherParts = message.parts.filter(p => p.type !== 'text');

        let html = '';

        // 1. Get all raw text and render it once.
        const rawText = getTextFromParts(textParts);
        contentDiv.dataset.rawText = rawText; // Store the raw text
        if (rawText) {
            html += marked.parse(rawText);
        }

        // 2. Render all other parts individually.
        html += otherParts.map(renderPart).join('');

        contentDiv.innerHTML = html;
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

        // --- THIS IS THE CHANGE ---
        // Use the new rendering function that handles raw text.
        renderMessageContent(contentDiv, message);

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

        if (lastMessageElement && lastMessageElement.dataset.role === message.role && message.role !== 'user') {
            const contentDiv = lastMessageElement.querySelector('.content');

            // --- THIS IS THE FIX ---
            // 1. Get the old raw text from the dataset.
            const oldRawText = contentDiv.dataset.rawText || '';

            // 2. Get the new text chunk from the incoming message.
            const newTextChunk = getTextFromParts(message.parts);

            // 3. Combine them to get the full raw text.
            const fullRawText = oldRawText + newTextChunk;

            // 4. Store the new full raw text back into the dataset.
            contentDiv.dataset.rawText = fullRawText;

            // 5. Re-render the entire content div with the full text.
            // We create a temporary message object for the renderer.
            // This also handles any non-text parts that might arrive.
            const existingNonTextParts = Array.from(contentDiv.querySelectorAll('.accordion'))
                .map(() => ({type: 'non-text-placeholder'})); // simplified for logic

            const updatedMessage = {
                parts: [
                    ...existingNonTextParts, // keep existing non-text parts
                    ...message.parts.filter(p => p.type !== 'text'), // add new non-text parts
                    {type: 'text', text: fullRawText} // add the complete text block
                ]
            };
            renderMessageContent(contentDiv, updatedMessage);

        } else {
            appendMessage(message);
        }

        scrollToBottom();
    };

    // ... The rest of the file (loadHistory, handleFormSubmit, etc.) remains unchanged ...

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
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            chatForm.dispatchEvent(new Event('submit'));
        }
    });

    loadHistory();
});