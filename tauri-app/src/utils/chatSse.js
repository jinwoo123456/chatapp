export function subscribeChat(roomId, onMessage) {
    const url = `http://localhost:3100/api/chat/subscribe?room_id=${encodeURIComponent(roomId)}`;
    const eventSource = new EventSource(url);
    eventSource.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            if (data.room_id === roomId) {
                onMessage(data);
            }
        } catch (e) {
            // ignore
        }
    };
    return eventSource;
}
