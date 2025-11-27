import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { Details } from './details';
import { Stats } from './pages/stats/stats';
import { Files } from './pages/files/files';



@NgModule({
  declarations: [],
  imports: [
    RouterModule.forChild([
      { path: '', component: Details, children: [
        { path: '', component: Stats },
        { path: 'files', component: Files }
      ] }
    ])
  ]
})
export class DetailsModule { }
