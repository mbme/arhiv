const NOTIFICATION_TIMEOUT_MS = 3400;

type NotificationType = 'info' | 'error';

export function renderNotification(message: string, notificationType: NotificationType = 'info'): void {
  const rootEl = document.getElementById('notification-root');
  if (!rootEl) {
    throw new Error('notification root el not found');
  }

  const domEl = document.createElement('div');
  domEl.innerText = message;
  domEl.dataset[notificationType] = '';
  domEl.onclick = () => {
    domEl.remove();
  };

  rootEl.appendChild(domEl);

  setTimeout(() => {
    domEl.remove();
  }, NOTIFICATION_TIMEOUT_MS);
}
