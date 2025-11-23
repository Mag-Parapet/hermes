import { createFeature, createReducer, on } from '@ngrx/store';
import { setLoading } from './loading.actions';

export const initialState: boolean = false;

export const loadingFeature = createFeature({
  name: 'loading',
  reducer: createReducer(
    initialState,

    on(setLoading, (state, action) => action.state)
  ),
});

export const { name, reducer, selectLoadingState } = loadingFeature;
