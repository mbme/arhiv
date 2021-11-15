type NotificationType = 'info' | 'error';

const NOTIFICATION_TIMEOUT_MS: Record<NotificationType, number> = {
  'info': 3400,
  'error': 10000,
};

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
  }, NOTIFICATION_TIMEOUT_MS[notificationType]);
}
