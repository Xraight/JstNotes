<script lang="ts">
  import { onMount } from 'svelte';
  import { calendarStore } from '../stores/calendar';
  import { noteStore } from '../stores/notes';

  let events = $derived(calendarStore.events);
  let year = $state(new Date().getFullYear());
  let month = $state(new Date().getMonth() + 1);
  let today = $state(new Date());
  let selectedDay = $state<number | null>(null);
  let creating = $state(false);

  let newTitle = $state('');
  let newDesc = $state('');
  let newNoteIds = $state<Set<string>>(new Set());
  let noteSearch = $state('');
  let dialogEl = $state<HTMLDialogElement>();
  let dialogX = $state(0);
  let dialogY = $state(0);
  let dragState = $state<{ ox: number; oy: number } | null>(null);

  let noteById = $derived(new Map(noteStore.notes.map(n => [n.id, n])));

  let filteredNotes = $derived.by(() => {
    if (!noteSearch.trim()) return [];
    const q = noteSearch.toLowerCase();
    return noteStore.notes.filter(n => n.title.toLowerCase().includes(q)).slice(0, 30);
  });

  function getNotePath(id: string): string {
    const parts: string[] = [];
    let current = noteById.get(id);
    while (current) {
      parts.unshift(current.title);
      current = current.parent_id ? noteById.get(current.parent_id) : undefined;
    }
    return parts.join(' ▸ ');
  }

  function getDescendants(id: string): string[] {
    const result: string[] = [];
    function walk(parentId: string) {
      for (const n of noteStore.notes) {
        if (n.parent_id === parentId) {
          result.push(n.id);
          walk(n.id);
        }
      }
    }
    walk(id);
    return result;
  }

  function hasChildren(id: string): boolean {
    return noteStore.notes.some(n => n.parent_id === id);
  }

  function childrenCount(id: string): number {
    return getDescendants(id).length;
  }

  let monthName = $derived([
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ][month - 1]);

  let daysInMonth = $derived(new Date(year, month, 0).getDate());
  let firstDayOfWeek = $derived(new Date(year, month - 1, 1).getDay());
  let gridDays = $derived(daysInMonth + firstDayOfWeek);

  let dayEvents = $derived.by(() => {
    if (!selectedDay) return [];
    return calendarStore.getEventsForDay(selectedDay);
  });

  let allNoteTitles = $derived.by(() => {
    const m = new Map<string, string>();
    for (const n of noteStore.notes) m.set(n.id, n.title);
    return m;
  });

  let expandedEvent = $state<string | null>(null);
  let calendarRoot = $state<HTMLDivElement>();

  onMount(() => {
    if (noteStore.notes.length === 0) noteStore.loadNoteTitles();
    calendarStore.loadEvents(year, month);
  });

  function prevMonth() {
    if (month === 1) { year--; month = 12; }
    else { month--; }
    selectedDay = null;
    creating = false;
    calendarStore.loadEvents(year, month);
  }

  function nextMonth() {
    if (month === 12) { year++; month = 1; }
    else { month++; }
    selectedDay = null;
    creating = false;
    calendarStore.loadEvents(year, month);
  }

  function hasEvent(day: number): boolean {
    return calendarStore.getEventsForDay(day).length > 0;
  }

  function selectDay(day: number) {
    selectedDay = day;
    creating = false;
    expandedEvent = null;
  }

  function toggleCreate() {
    creating = !creating;
    if (!creating) { newTitle = ''; newDesc = ''; newNoteIds = new Set(); }
  }

  function openNotePicker() {
    noteSearch = '';
    dialogX = Math.max(0, (window.innerWidth - 360) / 2);
    dialogY = Math.max(0, (window.innerHeight - 400) / 2);
    dialogEl?.showModal();
  }

  function closeNotePicker() {
    dialogEl?.close();
  }

  function startDrag(e: MouseEvent) {
    dragState = { ox: e.clientX - dialogX, oy: e.clientY - dialogY };
  }

  function doDrag(e: MouseEvent) {
    if (!dragState) return;
    dialogX = e.clientX - dragState.ox;
    dialogY = e.clientY - dragState.oy;
  }

  function stopDrag() {
    dragState = null;
  }

  function toggleNoteInPicker(id: string) {
    const next = new Set(newNoteIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    newNoteIds = next;
    noteSearch = '';
  }

  function selectSubtree(id: string) {
    const next = new Set(newNoteIds);
    const ids = [id, ...getDescendants(id)];
    // if all are already selected, deselect all; otherwise select all
    const allSelected = ids.every(i => next.has(i));
    for (const i of ids) {
      if (allSelected) next.delete(i); else next.add(i);
    }
    newNoteIds = next;
    noteSearch = '';
  }

  async function submitEvent() {
    if (!selectedDay || !newTitle.trim()) return;
    const date = `${year}-${String(month).padStart(2, '0')}-${String(selectedDay).padStart(2, '0')}`;
    await calendarStore.createEvent(date, newTitle.trim(), newDesc.trim(), [...newNoteIds]);
    newTitle = '';
    newDesc = '';
    newNoteIds = new Set();
    creating = false;
  }

  async function toggleCompleted(eventId: string, current: boolean) {
    const ev = events.find(e => e.id === eventId);
    if (ev) await calendarStore.updateEvent(eventId, ev.title, ev.description, !current);
  }

  async function deleteEvent(eventId: string) {
    if (confirm('Delete this event?')) {
      await calendarStore.deleteEvent(eventId);
      expandedEvent = null;
    }
  }

  async function unlinkNote(eventId: string, noteId: string) {
    await calendarStore.unlinkNote(eventId, noteId);
  }
</script>

<div class="calendar" data-calendar bind:this={calendarRoot}>
  <div class="cal-header">
    <button class="nav-btn" onclick={prevMonth}>&#x25C0;</button>
    <span class="cal-title">{monthName} {year}</span>
    <button class="nav-btn" onclick={nextMonth}>&#x25B6;</button>
  </div>

  <div class="cal-grid">
    {#each ['S', 'M', 'T', 'W', 'T', 'F', 'S'] as d}
      <span class="cal-day-header">{d}</span>
    {/each}

    {#each Array(gridDays) as _, i}
      {#if i < firstDayOfWeek}
        <span class="cal-cell empty"></span>
      {:else}
        {@const day = i - firstDayOfWeek + 1}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <button class="cal-cell"
          class:today={day === today.getDate() && month === today.getMonth() + 1 && year === today.getFullYear()}
          class:selected={day === selectedDay}
          class:has-event={hasEvent(day)}
          onclick={() => selectDay(day)}>
          {day}
        </button>
      {/if}
    {/each}
  </div>

  {#if selectedDay}
    <div class="day-panel">
      <div class="day-panel-header">
        <strong>{monthName} {selectedDay}</strong>
        <button class="add-btn" onclick={toggleCreate}>
          {creating ? 'Cancel' : '+ Event'}
        </button>
      </div>

      {#if creating}
        <form class="create-form" onsubmit={(e) => { e.preventDefault(); submitEvent(); }}>
          <input type="text" bind:value={newTitle} placeholder="Event title..." class="ev-input" />
          <input type="text" bind:value={newDesc} placeholder="Description (optional)" class="ev-input" />
          <div class="note-picker">
            <button type="button" class="picker-btn" onclick={openNotePicker}>
              Link Notes ({newNoteIds.size})
            </button>
          </div>

          <dialog bind:this={dialogEl} class="note-dialog" style="top: {dialogY}px; left: {dialogX}px;"
            onmousemove={doDrag} onmouseup={stopDrag} onmouseleave={stopDrag}
            onclick={(e) => { if (e.target === dialogEl) closeNotePicker(); }}>
            <div class="dialog-form">
              <div class="dialog-header" onmousedown={startDrag}>
                <strong>Link Notes</strong>
                <button type="button" class="dialog-close" onclick={closeNotePicker}>×</button>
              </div>
              <input type="text" class="dialog-search" bind:value={noteSearch}
                placeholder="Search notes by title..." />
              {#if noteSearch && filteredNotes.length > 0}
                <div class="dialog-results">
                  {#each filteredNotes as n}
                    <div class="dialog-item-group">
                      <label class="dialog-item" class:selected={newNoteIds.has(n.id)}>
                        <input type="checkbox" checked={newNoteIds.has(n.id)}
                          onchange={() => toggleNoteInPicker(n.id)} />
                        <div class="dialog-item-info">
                          <span class="dialog-item-title">{n.title}</span>
                          <span class="dialog-item-path">{getNotePath(n.id)}</span>
                        </div>
                      </label>
                      {#if hasChildren(n.id)}
                        <button type="button" class="dialog-subtree"
                          onclick={() => selectSubtree(n.id)}
                          title="Select this note and all {childrenCount(n.id)} descendants">
                          ↳{childrenCount(n.id)}
                        </button>
                      {/if}
                    </div>
                  {/each}
                </div>
              {:else if noteSearch && filteredNotes.length === 0}
                <div class="dialog-empty">No notes match "{noteSearch}"</div>
              {/if}
              {#if newNoteIds.size > 0}
                <div class="dialog-tags">
                  {#each [...newNoteIds] as nid}
                    {@const nt = noteStore.notes.find(n => n.id === nid)}
                    <span class="dialog-tag">
                      {nt?.title || 'Unknown'}
                      <button type="button" class="tag-remove" onclick={() => toggleNoteInPicker(nid)}>×</button>
                    </span>
                  {/each}
                </div>
              {/if}
              <div class="dialog-actions">
                <span class="dialog-count">{newNoteIds.size} selected</span>
                <button type="button" class="dialog-done" onclick={closeNotePicker}>Done</button>
              </div>
            </div>
          </dialog>
          <button type="submit" class="save-btn" disabled={!newTitle.trim()}>Save Event</button>
        </form>
      {/if}

      {#if dayEvents.length === 0 && !creating}
        <div class="empty-day">No events</div>
      {:else if !creating}
        <ul class="event-list">
          {#each dayEvents as ev}
            <li class="event-item" class:completed={ev.completed}>
              <button class="check-btn" onclick={() => toggleCompleted(ev.id, ev.completed)}>
                {ev.completed ? '✓' : '○'}
              </button>
              <button class="event-title" onclick={() => expandedEvent = expandedEvent === ev.id ? null : ev.id}>
                {ev.title}
              </button>
              <button class="del-btn" onclick={() => deleteEvent(ev.id)}>×</button>

              {#if expandedEvent === ev.id}
                <div class="event-detail">
                  {#if ev.description}
                    <p class="ev-desc">{ev.description}</p>
                  {/if}
                  <div class="ev-notes">
                    <strong>Linked notes ({ev.note_ids.length})</strong>
                    {#if ev.note_ids.length === 0}
                      <span class="no-notes">No linked notes</span>
                    {:else}
                      {#each ev.note_ids as nid}
                        <div class="ev-note-row">
                          <button class="ev-note-link" onclick={() => noteStore.selectNote(nid)}>
                            {allNoteTitles.get(nid) || 'Unknown'}
                          </button>
                          <button class="unlink-btn" onclick={() => unlinkNote(ev.id, nid)}>–</button>
                        </div>
                      {/each}
                    {/if}
                  </div>
                </div>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>

<style>
  .calendar {
    border-top: 1px solid var(--border);
    padding: 10px 12px;
    font-size: 12px;
    font-family: var(--font-sans);
    overflow-y: auto;
    overflow-x: hidden;
    flex-shrink: 0;
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
  }
  .cal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .cal-title {
    font-weight: 600;
    font-size: 13px;
  }
  .nav-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
    padding: 2px 6px;
    border-radius: 4px;
  }
  .nav-btn:hover {
    background: var(--accent);
    color: var(--text-primary);
  }
  .cal-grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 2px;
  }
  .cal-day-header {
    text-align: center;
    font-size: 10px;
    color: var(--text-secondary);
    font-weight: 600;
    padding: 2px 0;
  }
  .cal-cell {
    aspect-ratio: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    font-size: 12px;
    font-family: inherit;
    cursor: pointer;
    border: none;
    background: none;
    color: var(--text-primary);
    padding: 0;
  }
  .cal-cell:hover { background: var(--accent); }
  .cal-cell.today { font-weight: 700; color: var(--highlight); }
  .cal-cell.selected { background: var(--highlight); color: #fff; }
  .cal-cell.has-event { text-decoration: underline; text-underline-offset: 2px; text-decoration-color: var(--highlight); text-decoration-thickness: 2px; }
  .cal-cell.empty { cursor: default; }

  .day-panel { margin-top: 8px; border-top: 1px solid var(--border); padding-top: 8px; }
  .day-panel-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px; }
  .add-btn { background: none; border: 1px solid var(--border); border-radius: 4px; padding: 2px 8px; font-size: 11px; font-family: inherit; cursor: pointer; color: var(--text-secondary); }
  .add-btn:hover { color: var(--text-primary); border-color: var(--highlight); }

  .create-form { display: flex; flex-direction: column; gap: 6px; margin-bottom: 8px; max-width: 100%; }
  .ev-input { background: var(--bg-primary); border: 1px solid var(--border); border-radius: 4px; padding: 5px 8px; color: var(--text-primary); font-size: 12px; font-family: inherit; width: 100%; max-width: 100%; box-sizing: border-box; }
  .picker-btn { background: var(--accent); border: none; border-radius: 4px; padding: 4px 10px; font-size: 11px; font-family: inherit; cursor: pointer; color: var(--text-primary); }
  .picker-btn:hover { background: var(--highlight); }

  .note-dialog { border: 1px solid var(--border); border-radius: 12px; background: var(--bg-primary); color: var(--text-primary); padding: 0; width: 380px; max-width: 90vw; box-shadow: 0 8px 32px rgba(0,0,0,0.4); position: fixed; margin: 0; font-family: var(--font-sans); }
  .note-dialog::backdrop { background: rgba(0,0,0,0.4); }
  .dialog-form { padding: 16px; display: flex; flex-direction: column; gap: 10px; font-family: inherit; }
  .dialog-header { display: flex; align-items: center; justify-content: space-between; cursor: grab; user-select: none; padding: 4px 0; font-family: inherit; }
  .dialog-header:active { cursor: grabbing; }
  .dialog-header strong { font-size: 14px; }
  .dialog-close { background: none; border: none; color: var(--text-secondary); cursor: pointer; font-size: 18px; padding: 0; line-height: 1; font-family: inherit; }
  .dialog-close:hover { color: var(--text-primary); }
  .dialog-search { background: var(--bg-secondary); border: 1px solid var(--border); border-radius: 6px; padding: 8px 12px; color: var(--text-primary); font-size: 13px; font-family: inherit; width: 100%; }
  .dialog-results { max-height: 240px; overflow-y: auto; display: flex; flex-direction: column; gap: 2px; }
  .dialog-item-group { display: flex; align-items: flex-start; gap: 4px; padding: 4px 4px; border-radius: 6px; }
  .dialog-item-group:hover { background: var(--accent); }
  .dialog-item { display: flex; align-items: flex-start; gap: 8px; flex: 1; cursor: pointer; padding: 2px 0; font-size: 13px; font-family: inherit; }
  .dialog-item input { margin-top: 3px; font-family: inherit; }
  .dialog-item-info { display: flex; flex-direction: column; gap: 1px; }
  .dialog-item-title { font-size: 13px; }
  .dialog-item-path { font-size: 10px; color: var(--text-secondary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 280px; }
  .dialog-subtree { background: none; border: 1px solid var(--border); border-radius: 4px; padding: 2px 6px; font-size: 10px; font-family: inherit; cursor: pointer; color: var(--text-secondary); white-space: nowrap; flex-shrink: 0; margin-top: 2px; }
  .dialog-subtree:hover { border-color: var(--highlight); color: var(--highlight); }
  .dialog-empty { color: var(--text-secondary); font-style: italic; font-size: 12px; padding: 8px; text-align: center; }
  .dialog-tags { display: flex; flex-wrap: wrap; gap: 4px; }
  .dialog-tag { background: var(--accent); border-radius: 4px; padding: 3px 8px; font-size: 12px; display: inline-flex; align-items: center; gap: 4px; font-family: inherit; }
  .tag-remove { background: none; border: none; color: var(--text-secondary); cursor: pointer; font-size: 14px; padding: 0; line-height: 1; font-family: inherit; }
  .tag-remove:hover { color: var(--highlight); }
  .dialog-actions { display: flex; align-items: center; justify-content: space-between; }
  .dialog-count { font-size: 12px; color: var(--text-secondary); }
  .dialog-done { background: var(--highlight); border: none; border-radius: 6px; padding: 6px 20px; font-size: 13px; font-family: inherit; color: #fff; cursor: pointer; }
  .dialog-done:hover { opacity: 0.9; }
  .save-btn { background: var(--highlight); border: none; border-radius: 4px; padding: 5px 12px; font-size: 11px; font-family: inherit; color: #fff; cursor: pointer; }
  .save-btn:disabled { opacity: 0.4; cursor: default; }

  .empty-day { color: var(--text-secondary); font-style: italic; font-size: 11px; padding: 4px 0; }

  .event-list { list-style: none; margin: 0; padding: 0; }
  .event-item { border: 1px solid var(--border); border-radius: 6px; padding: 6px 8px; margin-bottom: 4px; }
  .event-item.completed { opacity: 0.5; }
  .event-item { display: flex; align-items: flex-start; gap: 6px; flex-wrap: wrap; }
  .check-btn { background: none; border: none; cursor: pointer; font-size: 14px; padding: 0; color: var(--text-secondary); flex-shrink: 0; margin-top: 1px; font-family: inherit; }
  .check-btn:hover { color: var(--highlight); }
  .event-title { flex: 1; font-size: 12px; cursor: pointer; min-width: 0; background: none; border: none; color: inherit; padding: 0; text-align: left; font-family: inherit; }
  .del-btn { background: none; border: none; color: var(--text-secondary); cursor: pointer; font-size: 14px; padding: 0 2px; opacity: 0; font-family: inherit; }
  .event-item:hover .del-btn { opacity: 1; }
  .del-btn:hover { color: var(--highlight); }

  .event-detail { width: 100%; padding: 6px 0 2px; border-top: 1px solid var(--border); margin-top: 4px; font-size: 11px; }
  .ev-desc { color: var(--text-secondary); margin: 0 0 6px; }
  .ev-notes { }
  .ev-notes strong { font-size: 11px; display: block; margin-bottom: 4px; }
  .no-notes { color: var(--text-secondary); font-style: italic; font-size: 11px; }
  .ev-note-row { display: flex; align-items: center; gap: 4px; padding: 2px 0; }
  .ev-note-link { background: var(--accent); border: none; border-radius: 3px; padding: 1px 6px; font-size: 11px; font-family: inherit; cursor: pointer; color: var(--text-primary); }
  .ev-note-link:hover { background: var(--highlight); }
  .unlink-btn { background: none; border: none; color: var(--text-secondary); cursor: pointer; font-size: 12px; padding: 0 2px; font-family: inherit; }
  .unlink-btn:hover { color: var(--highlight); }
</style>
