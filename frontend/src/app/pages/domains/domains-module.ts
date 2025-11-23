import { NgModule } from '@angular/core';
import { RouterModule } from '@angular/router';
import { Domains } from './domains';
import { Add } from './add/add';
import { Details } from './details/details';
import { Edit } from './edit/edit';



@NgModule({
  declarations: [],
  imports: [
    RouterModule.forChild([
      { path: '', component: Domains },
      { path: 'add', component: Add },
      { path: ':id', component: Details },
      { path: ':id/edit', component: Edit },
    ])
  ]
})
export class DomainsModule { }
