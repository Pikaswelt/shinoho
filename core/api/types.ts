/**
 * Shinoho API v0: Tab & Navigation Interfaces
 * Schnittstelle Kern <-> UI gemaess PLAN.md und AGENTS.md §5
 */

export type TabId = string;

export interface TabInfo {
  id: TabId;
  title: string;
  url: string;
  isLoading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
  isPinned: boolean;
  isMuted: boolean;
  favicon?: string;
  createdAt: number;
}

export interface CreateTabRequest {
  url?: string;
  activate?: boolean;
  isPinned?: boolean;
}

export interface NavigateTabRequest {
  tabId: TabId;
  url: string;
}

export interface TabActionRequest {
  tabId: TabId;
}

export interface ReloadTabRequest {
  tabId: TabId;
  ignoreCache?: boolean;
}

export type TabEventType =
  | 'tab-created'
  | 'tab-updated'
  | 'tab-closed'
  | 'tab-activated';

export interface TabCreatedEvent {
  type: 'tab-created';
  tab: TabInfo;
}

export interface TabUpdatedEvent {
  type: 'tab-updated';
  tab: TabInfo;
}

export interface TabClosedEvent {
  type: 'tab-closed';
  tabId: TabId;
}

export interface TabActivatedEvent {
  type: 'tab-activated';
  tabId: TabId;
}

export type TabEvent =
  | TabCreatedEvent
  | TabUpdatedEvent
  | TabClosedEvent
  | TabActivatedEvent;

/**
 * Tauri Command Names for Core Tab Management
 */
export const TAB_COMMANDS = {
  CREATE_TAB: 'create_tab',
  CLOSE_TAB: 'close_tab',
  ACTIVATE_TAB: 'activate_tab',
  NAVIGATE_TAB: 'navigate_tab',
  GO_BACK: 'go_back',
  GO_FORWARD: 'go_forward',
  RELOAD_TAB: 'reload_tab',
  GET_TABS: 'get_tabs',
} as const;

/**
 * Tauri Event Names
 */
export const TAB_EVENTS = {
  TAB_CREATED: 'shinoho://tab-created',
  TAB_UPDATED: 'shinoho://tab-updated',
  TAB_CLOSED: 'shinoho://tab-closed',
  TAB_ACTIVATED: 'shinoho://tab-activated',
} as const;
