import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'

const loading = document.getElementById('loading');
if (loading) loading.style.display = 'none';

const target = document.getElementById('app')!;
target.innerHTML = '';

const app = mount(App, { target });

export default app
