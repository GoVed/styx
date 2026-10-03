import React from 'react';
import { TelemetryStrip } from './components/TelemetryStrip';
import { ChatView } from './components/ChatView';
import { MemoryHub } from './components/MemoryHub';
import { ModelManager } from './components/ModelManager';
import { ToolHub } from './components/ToolHub';
import { AuditDrawer } from './components/AuditDrawer';
import { AccessGate } from './components/AccessGate';
import { SettingsHub } from './components/SettingsHub';
import { TopNavBar } from './components/navigation/TopNavBar';
import { MobileNav } from './components/navigation/MobileNav';
import { OnboardingBanner } from './components/navigation/OnboardingBanner';
import { SubViewBackBanner } from './components/navigation/SubViewBackBanner';
import { NotificationToast } from './components/navigation/NotificationToast';
import { useSyndaeApp } from './hooks/useSyndaeApp';

export const App: React.FC = () => {
  const {
    authStatus,
    isAuthenticated,
    authChecking,
    activeTab,
    setActiveTab,
    showTelemetryStrip,
    setShowTelemetryStrip,
    telemetry,
    pendingTickets,
    auditOpen,
    setAuditOpen,
    activeNotification,
    dismissNotification,
    sessions,
    activeSession,
    setActiveSession,
    messages,
    liveThought,
    liveStreamingText,
    liveToolExecutions,
    isStreaming,
    chatError,
    setChatError,
    memoryFiles,
    containers,
    presets,
    modelConfigs,
    mcpServers,
    mcpTools,
    auditEvents,
    loadMessages,
    handleCreateSession,
    handleDeleteSession,
    handleSendPrompt,
    handleResolveTicket,
    handleSelectActiveModel,
    handleUpdatePolicy,
    handleAuthenticated,
    handleLock,
    handleStartOnboarding,
    handleDismissOnboarding,
    refreshMemoryFiles,
    refreshModelData,
    refreshToolsData,
    refreshAuditData,
  } = useSyndaeApp();

  if (authChecking) {
    return (
      <div className="h-screen w-screen bg-zinc-950 flex flex-col items-center justify-center font-mono text-zinc-400">
        <div className="w-8 h-8 border-2 border-emerald-500 border-t-transparent rounded-full animate-spin mb-4" />
        <span className="text-xs uppercase tracking-widest text-emerald-400">Initializing Enclave...</span>
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <AccessGate
        status={
          authStatus || {
            initialized: false,
            onboarded: false,
            operator_name: null,
            requires_auth: true,
          }
        }
        onAuthenticated={handleAuthenticated}
      />
    );
  }

  return (
    <div
      className="h-screen w-screen w-full max-w-full overflow-hidden bg-syndae-950 text-slate-100 flex flex-col font-sans select-none"
      style={{ height: 'var(--app-height, 100svh)', maxHeight: 'var(--app-height, 100svh)' }}
    >
      {/* 1. Optional Hardware Vitals Strip */}
      {showTelemetryStrip && (
        <TelemetryStrip
          telemetry={telemetry}
          pendingApprovalsCount={pendingTickets.length}
          onOpenApprovals={() => setActiveTab('chat')}
          onToggleAudit={() => setAuditOpen(!auditOpen)}
          auditOpen={auditOpen}
        />
      )}

      {/* Onboarding Banner if not yet onboarded */}
      {authStatus && !authStatus.onboarded && (
        <OnboardingBanner
          onStartOnboarding={handleStartOnboarding}
          onDismissOnboarding={handleDismissOnboarding}
        />
      )}

      {/* Main Mission Control Tabs Navigation - on mobile Chat, TopNavBar is hidden so ChatView has full vertical space with its unified top bar */}
      <div className={`flex-shrink-0 ${activeTab === 'chat' ? 'hidden md:block' : 'block'}`}>
        <TopNavBar
          activeTab={activeTab}
          authStatus={authStatus}
          telemetry={telemetry}
          pendingTickets={pendingTickets}
          showTelemetryStrip={showTelemetryStrip}
          onSelectTab={setActiveTab}
          onToggleTelemetryStrip={() => setShowTelemetryStrip(!showTelemetryStrip)}
          onLock={handleLock}
        />
      </div>

      {/* Sub-view Back Banner when drilled down into a settings section */}
      {(activeTab === 'models' || activeTab === 'memory' || activeTab === 'tools') && (
        <SubViewBackBanner
          activeTab={activeTab}
          onBackToSettings={() => setActiveTab('settings')}
        />
      )}

      {/* Dynamic Agent Alert / Notification Toast */}
      <NotificationToast
        notification={activeNotification}
        onDismiss={dismissNotification}
        onOpenSession={sessionId => {
          const matched = sessions.find(s => s.id === sessionId);
          if (matched) {
            setActiveSession(matched);
          } else {
            setActiveSession({ id: sessionId } as any);
          }
          setActiveTab('chat');
        }}
      />

      {/* 2. Main Content Area */}
      <main className="flex-1 min-h-0 relative overflow-hidden bg-syndae-950 flex flex-col">
        {activeTab === 'chat' && (
          <ChatView
            sessions={sessions}
            activeSession={activeSession}
            onSelectSession={setActiveSession}
            onCreateSession={handleCreateSession}
            onDeleteSession={handleDeleteSession}
            pendingTickets={pendingTickets}
            onResolveTicket={handleResolveTicket}
            memoryFiles={memoryFiles}
            onSendPrompt={handleSendPrompt}
            liveThought={liveThought}
            liveStreamingText={liveStreamingText}
            liveToolExecutions={liveToolExecutions}
            isStreaming={isStreaming}
            chatError={chatError}
            onClearChatError={() => setChatError(null)}
            messages={messages}
            onReloadMessages={() => activeSession && loadMessages(activeSession.id)}
            onNavigateToSettings={() => setActiveTab('settings')}
          />
        )}

        {activeTab === 'settings' && (
          <SettingsHub
            telemetry={telemetry}
            authStatus={authStatus}
            modelConfigs={modelConfigs}
            containers={containers}
            mcpTools={mcpTools}
            pendingTicketsCount={pendingTickets.length}
            onNavigate={(view) => {
              if (view === 'diagnostics') {
                setActiveTab('settings');
              } else {
                setActiveTab(view as any);
              }
            }}
            onNavigateSection={(section) => setActiveTab(section as any)}
            onLock={handleLock}
            onStartOnboarding={handleStartOnboarding}
            onOpenAudit={() => setAuditOpen(true)}
            onBackToChat={() => setActiveTab('chat')}
            onPasswordChanged={(token) => authStatus && handleAuthenticated(token, authStatus)}
          />
        )}

        {activeTab === 'models' && (
          <ModelManager
            containers={containers}
            presets={presets}
            modelConfigs={modelConfigs}
            telemetry={telemetry}
            onRefresh={refreshModelData}
            onSelectActiveModel={handleSelectActiveModel}
            onNavigateToChat={() => setActiveTab('chat')}
          />
        )}

        {activeTab === 'memory' && (
          <MemoryHub
            files={memoryFiles}
            onRefreshFiles={refreshMemoryFiles}
          />
        )}

        {activeTab === 'tools' && (
          <ToolHub
            servers={mcpServers}
            tools={mcpTools}
            onRefresh={refreshToolsData}
            onUpdatePolicy={handleUpdatePolicy}
          />
        )}

        {/* 3. Global Slide-over Audit Trail Drawer */}
        <AuditDrawer
          isOpen={auditOpen}
          onClose={() => setAuditOpen(false)}
          auditEvents={auditEvents}
          onRefresh={refreshAuditData}
        />
      </main>

      {/* 4. Mobile Bottom Navigation Bar (Touch-friendly for phones) - only shown when outside chat */}
      {activeTab !== 'chat' && (
        <MobileNav
          activeTab={activeTab}
          pendingTickets={pendingTickets}
          onSelectTab={setActiveTab}
        />
      )}
    </div>
  );
};

export default App;
