import { createFeature, createReducer, on } from '@ngrx/store';
import { toggleSidenav } from './sidenav.actions';

export const initialState: boolean = false;

export const sidenavFeature = createFeature({ 
  name: 'sidenav',
  reducer: createReducer(
    initialState,
    on(toggleSidenav, (state) => {
      const newState = !state;
      localStorage.setItem('sidenav-collapsed', String(newState));
      return newState;
    })
  )
});

export const { name, reducer, selectSidenavState } = sidenavFeature;