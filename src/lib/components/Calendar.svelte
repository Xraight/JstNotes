<script lang="ts">
  import { onMount } from 'svelte';
  import { calendarStore } from '../stores/calendar';
  import { noteStore } from '../stores/notes';

  let entries = $derived(calendarStore.entries);
  let year = $state(new Date().getFullYear());
  let month = $state(new Date().getMonth() + 1);
  let today = $state(new Date());
  let selectedDay = $state<number | null>(null);
  let editingDay = $state<number | null>(null);
  let newTitle = $state('');

  let monthName = $derived([
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ][month - 1]);

  let daysInMonth = $derived(new Date(year, month, 0).getDate());
  let firstDayOfWeek = $derived(new Date(year, month - 1, 1).getDay());
  let gridDays = $derived(daysInMonth + firstDayOfWeek);

  let dayEntries = $derived.by(() => {
    if (!selectedDay) return [];
    return calendarStore.getEntriesForDay(selectedDay);
  });

  onMount(() => {
    calendarStore.loadEntries(year, month);
  });

  function prevMonth() {
    if (month === 1) { year--; month = 12; }
    else { month--; }
    selectedDay = null;
    editingDay = null;
    calendarStore.loadEntries(year, month);
  }

  function nextMonth() {
    if (month === 12) { year++; month = 1; }
    else { month++; }
    selectedDay = null;
    editingDay = null;
    calendarStore.loadEntries(year, month);
  }

  function hasEntry(day: number): boolean {
    return calendarStore.getEntriesForDay(day).length > 0;
  }

  function selectDay(day: number) {
    selectedDay = day;
    editingDay = null;
  }

  function addEntry() {
    if (!selectedDay || !newTitle.trim()) return;
    const date = `${year}-${String(month).padStart(2, '0')}-${String(selectedDay).padStart(2, '0')}`;
    calendarStore.createEntry(date, newTitle.trim());
    newTitle = '';
  }

  function deleteEntry(id: string) {
    calendarStore.deleteEntry(id);
  }

  function linkNote(entryId: string, noteId: string) {
    calendarStore.updateEntry(entryId, '', noteId);
  }
</script>

<div class="calendar">
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
          class:has-entry={hasEntry(day)}
          onclick={() => selectDay(day)}>
          {day}
        </button>
      {/if}
    {/each}
  </div>

  {#if selectedDay}
    <div class="day-panel">
      <div class="day-panel-header">
        <strong>{monthName} {selectedDay}, {year}</strong>
        <button class="add-btn" onclick={() => editingDay = selectedDay}>+ Add</button>
      </div>

      {#if editingDay === selectedDay}
        <form class="add-form" onsubmit={(e) => { e.preventDefault(); addEntry(); }}>
          <input type="text" bind:value={newTitle} placeholder="Event title..." class="entry-input" />
          <button type="submit" class="save-btn">Save</button>
        </form>
      {/if}

      {#if dayEntries.length === 0}
        <div class="empty-day">No entries</div>
      {:else}
        <ul class="entry-list">
          {#each dayEntries as entry}
            <li class="entry-item">
              <span class="entry-title">{entry.title}</span>
              {#if entry.note_id}
                <button class="link-badge" onclick={() => noteStore.selectNote(entry.note_id!)}>
                  {entry.note_title || '📄'}
                </button>
              {/if}
              <button class="delete-btn" onclick={() => deleteEntry(entry.id)}>×</button>
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
    padding: 2px 6px;
    border-radius: 4px;
  }
  .nav-btn:hover {
    background: var(--accent);
    color: var(--text-primary);
  }
  .cal-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
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
    cursor: pointer;
    border: none;
    background: none;
    color: var(--text-primary);
    padding: 0;
  }
  .cal-cell:hover {
    background: var(--accent);
  }
  .cal-cell.today {
    font-weight: 700;
    color: var(--highlight);
  }
  .cal-cell.selected {
    background: var(--highlight);
    color: #fff;
  }
  .cal-cell.has-entry {
    text-decoration: underline;
    text-underline-offset: 2px;
    text-decoration-color: var(--highlight);
    text-decoration-thickness: 2px;
  }
  .cal-cell.empty {
    cursor: default;
  }
  .day-panel {
    margin-top: 8px;
    border-top: 1px solid var(--border);
    padding-top: 8px;
  }
  .day-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }
  .add-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 11px;
    cursor: pointer;
    color: var(--text-secondary);
  }
  .add-btn:hover {
    color: var(--text-primary);
    border-color: var(--highlight);
  }
  .add-form {
    display: flex;
    gap: 4px;
    margin-bottom: 6px;
  }
  .entry-input {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 4px 8px;
    color: var(--text-primary);
    font-size: 12px;
  }
  .save-btn {
    background: var(--highlight);
    border: none;
    border-radius: 4px;
    padding: 4px 10px;
    font-size: 11px;
    color: #fff;
    cursor: pointer;
  }
  .empty-day {
    color: var(--text-secondary);
    font-style: italic;
    font-size: 11px;
    padding: 4px 0;
  }
  .entry-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .entry-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 0;
    font-size: 12px;
  }
  .entry-title {
    flex: 1;
  }
  .link-badge {
    background: var(--accent);
    border: none;
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 10px;
    cursor: pointer;
    color: var(--text-primary);
  }
  .link-badge:hover {
    background: var(--highlight);
  }
  .delete-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 14px;
    padding: 0 2px;
    opacity: 0;
  }
  .entry-item:hover .delete-btn {
    opacity: 1;
  }
  .delete-btn:hover {
    color: var(--highlight);
  }
</style>
