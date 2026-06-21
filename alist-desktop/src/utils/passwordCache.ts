const ADMIN_PASSWORD_KEY = "yuhang-alisting:last-admin-password";

export function getCachedAdminPassword() {
  return localStorage.getItem(ADMIN_PASSWORD_KEY) ?? "";
}

export function setCachedAdminPassword(password: string) {
  const value = password.trim();

  if (value) {
    localStorage.setItem(ADMIN_PASSWORD_KEY, value);
  }
}
